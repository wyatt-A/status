use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use rand::Rng;
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
    pub fn file_check(&self, user_args:&StatusArgs, required_matches:&Vec<String>, base_runno:Option<String>) -> Status {
        use SignatureType::*;

        println!("running file check ...");

        //println!("stage label: {}",self.label);

        let re = Regex::new(r"(\$\{[[:alnum:]_]+\})").unwrap();

        // return the big disk that we are using on this system
        let big_disk = match &user_args.biggus_diskus() {
            Some((hostname,big_disk)) => {
                match utils::computer_name().as_str() == hostname.as_str() {
                    true => {
                        big_disk.to_string()
                    }
                    false => std::env::var("BIGGUS_DISKUS").expect("BIGGUS_DISKUS is not set")
                }

            }
            None => std::env::var("BIGGUS_DISKUS").expect("BIGGUS_DISKUS is not set")
        };

        // may have user arg BIGGUS_DISKUS
        // it an optional hostname : alternate_biggus
        // when it has no hostname, OR hostname matches this host, we use it.

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
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&base_runno.clone().unwrap_or("BASE_RUNNO".to_string()));
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

    fn status(&self, user_args:&StatusArgs, required_matches:&Vec<String>, base_runno: Option<String>) -> Status {

        println!("running stage status for {} ...",self.label);


        let hostname = utils::computer_name();

        let computers = match &self.preferred_computer {
            Some(computers) => {
                let mut h = HashSet::<&str>::new();
                for computer in computers {
                    h.insert(computer);
                }
                match h.contains(hostname.as_str()) {
                    true => None,
                    false => Some(computers.clone())
                }
            }
            None => {
                None
            }
        };

        let the_status = match &computers {

            Some(computers) => {

                println!("preferred computers found for {}: {:?}",self.label,computers);

                let mut temp_status = Status {
                    label: "dummy".to_string(),
                    progress: StatusType::NotStarted,
                    children: vec![]
                };

                for computer in computers {

                    // figure out where things are going on the remote computer
                    println!("getting home dir from {}",computer);
                    let mut cmd = Command::new("ssh");
                    cmd.args(vec![computer.as_str(),"echo",r#"$HOME"#]);
                    let o = cmd.output().expect("failed to run");
                    if !o.status.success(){
                        panic!("remote echo command failed");
                    }
                    let mut out = String::from_utf8(o.stdout.clone()).unwrap();
                    out.retain(|c| !c.is_whitespace());
                    let remote_dir = Path::new(&out).join(".pipe_status");

                    println!("remote dir = {:?}",remote_dir);


                    let local_home_dir = std::env::home_dir().expect("home dir cannot be fetched. Is the function deprecated?");

                    // random number for filename to out of overwrite paranoia
                    let mut rng = rand::thread_rng();
                    let n1: u8 = rng.gen();

                    let file_name = format!("{}{}",self.label.as_str(),n1.to_string().as_str());

                    let output_file = remote_dir.join(&file_name);
                    let local_file = local_home_dir.join(".pipe_status").join(&file_name);

                    let mut args = user_args.clone();

                    args.output_file = Some(output_file.clone());
                    args.config_dir = Some(remote_dir.join("pipe_configs"));
                    args.stage = Some(self.label.clone());
                    args.pipe_registry = Some(remote_dir.join("pipe_configs").join("pipe_registry"));

                    match &user_args.config_dir {
                        Some(conf_dir) => {
                            // todo!(use make temp to get a directory)

                            println!("sending pipe configurations to {}",computer);

                            // determine where we are putting the configs and status.json file on the remote
                            // computer


                            // make directory on the remote system
                            let mut cmd = Command::new("ssh");
                            cmd.args(vec![computer.as_str(),"mkdir",remote_dir.to_str().unwrap()]);
                            let o = cmd.output().expect("failed to run");
                            if !o.status.success(){
                                //panic!("remote mkdir command failed");
                            }

                            let output_file = remote_dir.join("status_out");


                            // send config files to remote
                            let mut cmd = Command::new("scp");
                            cmd.args(vec![
                                "-pr",
                                conf_dir.to_str().unwrap(),
                                &format!("{}:{}",computer,remote_dir.to_str().unwrap())
                            ]);
                            println!("running {:?}",cmd);
                            if !cmd.output().expect("failed to launch scp").status.success() {
                                panic!("scp failed");
                            }



                        }
                        None => {
                            println!("no configuration dir is set. Not sending to {}",computer);
                        }
                    }

                    println!("running remote status call on {}",computer);

                    // run remote check
                    let mut cmd = Command::new("ssh");
                    cmd.args(vec![
                        computer.as_str(),
                        "declare -x RUST_BACKTRACE=1;",
                        remote_dir.join("status").to_str().unwrap(),
                    ]);
                    cmd.args(args.to_vec());
                    println!("running {:?}",cmd);
                    let o = cmd.output().expect("failed to launch ssh");
                    let stdout = String::from_utf8(o.stdout.clone()).unwrap();
                    let stderr = String::from_utf8(o.stderr.clone()).unwrap();
                    println!("err = {}",stderr);
                    println!("output = {}",stdout);


                    // collect status)
                    //todo(define local TEMP status file in cool way)

                    println!("gathering output from {}",computer);

                    let mut cmd = Command::new("scp");
                    cmd.args(vec![
                        "-p",
                        &format!("{}:{}",computer.as_str(),output_file.with_extension("json").to_str().unwrap()),
                        &format!("{}", local_file.with_extension("json").to_str().unwrap())
                    ]);
                    println!("trying to run {:?}",cmd);
                    let o = cmd.output().expect("failed to launch scp");
                    if !o.status.success(){
                        panic!("scp failed");
                    }

                    println!("reading results ...");

                    let s = utils::read_to_string(&local_file,"json");
                    let stat:Status = serde_json::from_str(&s).expect("cannot deserialize struct");
                    temp_status = stat.children[0].clone();

                    std::fs::remove_file(&local_file.with_extension("json"));

                    match &temp_status.progress{
                        StatusType::NotStarted => {
                            println!("status returned {:?} ... checking next computer",StatusType::NotStarted)
                        }
                        _=> {
                            println!("found some progress, returning the status from {}",computer);
                            break
                        }
                    }

                }
                temp_status
            }
            None => {
                println!("no preferred computer set for {}.... running local file check",self.label);
                let stat = self.file_check(&user_args, &required_matches, base_runno);
                stat
            }
        };
        //println!("\tProgress {:?}",the_status.progress);
        the_status
    }
}
