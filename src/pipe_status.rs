use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize};
use regex::{Regex};
use crate::args::StatusArgs;
//se crate::client::UserArgs;
use crate::pipe_registry::{PIPE_REGISTRY_FILE, PipeRegistry};
use crate::stage::{SignatureType, Stage};
use crate::status_check::{Status, StatusCheck, StatusType};





#[derive(Serialize,Deserialize,Debug,Clone)]
struct StageStatus {
    val:f32,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
struct PipeStatus {
    stages:Vec<StageStatus>,
    val:f32,
}



#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipeStatusConfig {
    // a unit of work with a defined point completion
    pub label:String,
    pub preferred_computer:Option<Vec<String>>,
    pub stages:Vec<Stage>,
    // never should be loaded from disk, but set on load
    registry:Option<HashMap<String, PipeStatusConfig>>,
}

impl PipeStatusConfig {

    pub fn open(pipe_conf:&Path) -> Self {
        let string = utils::read_to_string(pipe_conf,"toml");
        let mut pipe_conf: PipeStatusConfig = toml::from_str(&string).expect("cannot deserialize pipe!");

        pipe_conf.stages.iter_mut().for_each(|stage|{
            if pipe_conf.preferred_computer.is_some(){
                stage.preferred_computer.get_or_insert(pipe_conf.preferred_computer.clone().unwrap());
            }
        });

        pipe_conf

    }

    pub fn to_hash(&self) -> BTreeMap<String,Stage> {
        let mut map = BTreeMap::<String,Stage>::new();
        for stage in &self.stages {
            map.insert(stage.label.clone(),stage.clone());
        }
        map
    }

    pub fn set_registry(&mut self,pipe_registry:&HashMap<String, PipeStatusConfig>) {
        self.registry = Some(pipe_registry.clone());
    }

    pub fn gen_template(pipe_conf:&Path) {
        let p = Self {
            label: String::from("what_this_pipe_is_called"),
            preferred_computer: None,
            stages: vec![
                Stage{
                    label: String::from("some_stage_name"),
                    preferred_computer:None,
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap(),
                    directory_pattern: "co_reg_${RUNNO}-inputs".to_string(),
                    signature_type:SignatureType::ManyToMany,
                    required_file_keywords:None,
                },
                Stage{
                    label: String::from("some_second_stage_name"),
                    preferred_computer:None,
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap(),
                    directory_pattern: "co_reg_${RUNNO}-results".to_string(),
                    signature_type:SignatureType::ManyToMany,
                    required_file_keywords:None,
                },
            ],
            registry: None
        };
        let str = toml::to_string(&p).expect("cannot deserialize struct");
        utils::write_to_file(pipe_conf,"txt",&str);
    }

}

// impl Stage {
//     pub fn regex(&self) -> Regex{
//         Regex::new(&self.completion_file_pattern).expect("invalid regex!")
//     }
// }

impl StatusCheck for PipeStatusConfig {
    fn status(&self,user_args:&StatusArgs ,required_matches: &Vec<String>, base_runno: Option<&str>) -> Status {

        let mut total_pipe_status = Status{
            label: self.label.clone(),
            progress: StatusType::NotStarted,
            children: vec![]
        };

        let stages = self.stages.clone();
        let stages_hash = self.to_hash();
        let stages:Vec<Stage> = match &user_args.stage {
            Some(stage_label) => {
                match stages_hash.get(stage_label.as_str()) {
                    Some(stage) => vec![stage.clone()],
                    _=> vec![]
                }
            }
            None => {
                stages.clone()
            }
        };

        let mut n_complete:f32 = 0.0;


        if stages.len() == 1 {
            println!("THE STAGE LENGTH IS 1!!!");
        }

        for stage in &stages {

            let mut stat = stage.status(user_args,required_matches,base_runno.clone());

            println!("status: {:?}",stat);

            match &stat.progress {
                StatusType::NotStarted => {
                    // if stage is pipe, recurse
                    if self.registry.clone().unwrap().get(stage.label.as_str()).is_some() {

                        let mut these_args = user_args.clone();
                        these_args.last_pipe = stage.label.clone();
                        //these_args.output_file = Some(PathBuf::from(r"$HOME/.spec_status_config/PIPENAME"));
                        these_args.output_file = Some(PathBuf::from(r"/Users/Wyatt/.spec_status_config/PIPENAME"));
                        these_args.stage = None;
                        //these_args.output_file =
                        let mut string_args = these_args.to_vec();
                        let this_exe = std::env::current_exe().unwrap();
                        let this_exe = this_exe.file_name().unwrap().to_str().unwrap();


                        let mut cmd = Command::new(this_exe);
                        cmd.args(string_args);

                        // let mut cmd = Command::new("ssh");
                        // string_args.insert(0,"localhost".to_string());
                        // string_args.insert(1,this_exe.to_owned());
                        // cmd.args(string_args);

                        if !cmd.output().expect(&format!("failed to launch {}",this_exe)).status.success() {

                            println!("{:?}",cmd);

                            panic!("recursive call failed");
                        }


                        //todo(load status)
                        let s = utils::read_to_string(&these_args.output_file.clone().unwrap(),"json");
                        stat = serde_json::from_str(s.as_str()).expect("cannot deserialize struct");
                        println!("statusP: {:?}",stat);
                    }
                }
                _=> {
                    println!("I AM NOT A PIPE!");

                }
            }


            //todo(add weighting to configs, and weight the progress value here)
            let stage_weight=1.0;

            let n_stages = stages.len() as f32;
            println!("n_stages  ={}",n_stages);

            let mut normalized_progress =  (stat.progress * stage_weight);// / stages.len() as f32;

            normalized_progress = normalized_progress/ n_stages;

            println!("normalized_progress  ={}",normalized_progress.to_float());

            total_pipe_status.progress = total_pipe_status.progress + (stat.progress * stage_weight) / stages.len() as f32;

            total_pipe_status.children.push(stat);
        }
        total_pipe_status
    }
}