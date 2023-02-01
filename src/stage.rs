use std::cell::RefCell;
use std::path::PathBuf;
use regex::Regex;
use crate::status_check::{Status, StatusCheck};
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


impl StatusCheck for Stage {
    fn status(&self,_user_args:&StatusArgs,required_matches:&Vec<String>,base_runno:Option<&str>) -> Status {
        use SignatureType::*;

        //self.directory_pattern


        //
        // is the host name matched by preferred computer?
        // let computer = match &self.preferred_computer {
        //     None => {
        //         vec![Computer::Local]
        //     }
        //     Some(preferred_hosts) => {
        //         // check and handle the case where the remote host is actually the local host
        //         preferred_hosts.iter().map(|host| Computer::Remote(host.clone())).collect()
        //     }
        // };





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


        println!("resolved directory pattern :{:?}",the_dir);

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

        let contents = std::fs::read_dir(&the_dir).expect(&format!("{} doesn't exist",the_dir));
        let mut included = vec![];
        for thing in contents {
            let tp = thing.unwrap();
            let file_name = tp.path().file_name().unwrap().to_string_lossy().into_owned();
            let file_path = tp.path().to_string_lossy().into_owned();
            if self.completion_file_pattern.is_match(&file_path) {
                included.push(file_name)
            }
        }

        println!("considered items: {:?}",included);

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
            Status::Complete
        } else if count == 0 {
            Status::NotStarted

        } else {
            Status::InProgress(count as f32 / required_matches.len() as f32)
        }

    }
}
