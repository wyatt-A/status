use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;
use crate::status_check::{Status, StatusCheck, StatusType};
use serde::{Serialize, Deserialize, Deserializer};
use crate::args::StatusArgs;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum SignatureType {
    Discrete,
    ManyToOne,
    ManyToMany,
    OneToMany,
    OneToOne,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Stage {
    pub label:String,
    pub preferred_computer:Option<Vec<String>>,
    #[serde(with = "serde_regex")]
    pub completion_file_pattern:Regex,
    pub directory_pattern:String,
    pub signature_type:SignatureType,
    pub required_file_keywords:Option<Vec<String>>,
}

impl Stage {
    fn file_check(&self,_user_args:&StatusArgs,required_matches:&Vec<String>,base_runno:Option<&str>) -> Status {
        use SignatureType::*;

        println!("stage label: {}",self.label);

        let re = Regex::new(r"(\$\{[[:alnum:]_]+\})").unwrap();
        let big_disk = std::env::var("BIGGUS_DISKUS").expect("BIGGUS_DISKUS is not set");

        let mut the_dir = self.directory_pattern.clone();
        re.captures_iter(&self.directory_pattern).for_each(|captures|{
            for cap_idx in 1..captures.len(){
                if captures[cap_idx].eq("${BIGGUS_DISKUS}") {
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&big_disk);
                }else if captures[cap_idx].eq("${PARAM0}") {
                    //todo(deal with 0 match?)
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&required_matches[0]);
                }else if captures[cap_idx].eq("${PREFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"diffusion");
                }
                else if captures[cap_idx].eq("${SUFFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }
                else if captures[cap_idx].eq("${PROGRAM}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"dsi_studio");
                }
                else if captures[cap_idx].eq("${BASE}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&base_runno.clone().unwrap_or("BASE_RUNNO"));
                }
                else if captures[cap_idx].eq("${SEP}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }else {
                    panic!("capture not recognized")
                }
            }
        });

        // directory should be valid now... if its missing we cant check?... or this stage has 0 progress
        // return not started?

        println!("\tresolved directory pattern :{:?}",the_dir);

        // trim required matches based on signature type=
        let required_matches = match &self.signature_type {
            ManyToMany => {
                required_matches.clone()
            }
            ManyToOne => {
                // we will assume only the base runno is involved in the match
                vec![base_runno.expect("base runno must be specified for ManyToOne signature type").to_string()]
            }

            OneToMany => {
                self.required_file_keywords.clone().expect("you need to specify required file keywords for OneToMany signature pattern")
            }

            OneToOne => {
                // relies on completion file pattern to filter to the ONLY thing which should match.
                vec![".".to_string()]
            }

            _=> {
                panic!("signature not implemented")
            }

        };

        let contents = match std::fs::read_dir(&the_dir) {
            Err(_) => return Status{
                label: self.label.clone(),
                progress: StatusType::NotStarted,
                children: vec![]
            },
            Ok(contents) => contents
        };

        let mut included = vec![];
        for thing in contents {
            let tp = thing.unwrap();
            let file_name = tp.path().file_name().unwrap().to_string_lossy().into_owned();
            let file_path = tp.path().to_string_lossy().into_owned();
            if self.completion_file_pattern.is_match(&file_path) {
                included.push(file_name)
            }
        }

        println!("\t\tconsidered items: {:?}",included);
        println!("\t\tpattern: {:?}",self.completion_file_pattern);

        included.sort();
        let glob = included.join("/");

        let mut count = 0;

        for rm in &required_matches {
            if Regex::new(&format!("(^|/).*{}.*($|/)",rm)).unwrap().is_match(&glob){
                count = count + 1;
                //println!("{}",rm);
            }
        }

        return if count == required_matches.len() {
            Status{
                label: self.label.clone(),
                progress: StatusType::Complete,
                children: vec![]
            }
        } else if count == 0 {
            Status{
                label: self.label.clone(),
                progress: StatusType::NotStarted,
                children: vec![]
            }
        } else {
            Status{
                label: self.label.clone(),
                progress: StatusType::InProgress(count as f32 / required_matches.len() as f32),
                children: vec![]
            }
        }
    }
}



impl StatusCheck for Stage {

    fn status(&self,user_args:&StatusArgs,required_matches:&Vec<String>,base_runno:Option<&str>) -> Status {
        let the_status = match &self.preferred_computer {
            Some(computers) => {
                let mut args = user_args.clone();
                args.output_file = Some(PathBuf::from(r"\$HOME/.spec_status_config_tmp/PIPENAME"));
                let remote_temp_dir = Path::new(r"\$HOME/.spec_status_config_tmp");
                args.config_dir = Some(remote_temp_dir.to_owned());
                args.stage = Some(self.label.clone());

                let mut temp_status = Status {
                    label: "dummy".to_string(),
                    progress: StatusType::NotStarted,
                    children: vec![]
                };

                for computer in computers {
                    match &user_args.config_dir {
                        Some(conf_dir) => {
                            // todo!(use make temp to get a directory)

                            let mut cmd = Command::new("scp");
                            cmd.args(vec![
                                "-pr",
                                &format!("{:?}", conf_dir),
                                computer.as_str(),
                                &format!(":{:?}", remote_temp_dir)
                            ]);

                            if !cmd.output().expect("failed to launch scp").status.success() {
                                panic!("scp failed");
                            }
                        }
                        None => {}
                    }

                    let bin_name = std::env::current_exe().unwrap();
                    let bin_name = bin_name.file_name().unwrap().to_str().unwrap();
                    // run remote check
                    Command::new("ssh").args(vec![
                        computer.as_str(),
                        bin_name,
                        args.to_string().as_str()
                    ]);
                    // collect status)
                    //todo(define local TEMP status file in cool way)
                    let local_status_file = Path::new(r"$HOME/.spec_tatus_config_tmp/incoming");
                    let mut cmd = Command::new("scp").args(vec![
                        "-p",
                        computer.as_str(),
                        &format!(":{:?}", args.config_dir),
                        &format!("{:?}", local_status_file)
                    ]);

                    let s = utils::read_to_string(local_status_file,"json");
                    let stat:Status = serde_json::from_str(&s).expect("cannot deserialize struct");
                    temp_status = stat.children[0].clone();

                    match &temp_status.progress{
                        StatusType::NotStarted => {}
                        _=> break
                    }

                }
                temp_status
            }
            None => {
                let stat = self.file_check(&user_args, &required_matches, base_runno);
                stat
            }
        };
        println!("\tProgress {:?}",the_status.progress);
        the_status
    }

}
