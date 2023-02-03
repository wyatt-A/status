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
use rand::Rng;

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
        let mut pipe_conf: PipeStatusConfig = toml::from_str(&string).expect(&format!("cannot deserialize {:?}",string));

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

impl StatusCheck for PipeStatusConfig {
    fn status(&self,user_args:&StatusArgs,required_matches: &Vec<String>, base_runno: Option<&str>) -> Status {

        // the complete pipeline status that will be updated and returned
        let mut total_pipe_status = Status{
            label: self.label.clone(),
            progress: StatusType::NotStarted,
            children: vec![]
        };

        // get copy of stages to make
        let stages = self.stages.clone();
        let stages_hash = self.to_hash();
        let mut stages:Vec<Stage> = match &user_args.stage {
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




        let reverse = true;
        let mut pipe_progress:f32 = 1.0;
        if reverse {
            stages.reverse();
        }

        //let mut n_complete:f32 = 0.0;
        for stage in &stages {

            // get the status for this stage assuming it is just a stage (not a pipe)
            let mut stat = stage.status(user_args,required_matches,base_runno.clone());

            match &stat.progress {


                StatusType::NotStarted => {
                    // if stage is pipe, recurse
                    // configure output status file and input args
                    if self.registry.clone().unwrap().get(stage.label.as_str()).is_some() {
                        let mut these_args = user_args.clone();
                        these_args.last_pipe = stage.label.clone();
                        //these_args.output_file = Some(PathBuf::from(r"$HOME/.spec_status_config/PIPENAME"));

                        let home_dir = std::env::home_dir().expect("home dir cannot be fetched. Is the function deprecated?");

                        // random number for filename to out of overwrite paranoia
                        let mut rng = rand::thread_rng();
                        let n1: u8 = rng.gen();

                        these_args.output_file = Some(home_dir.join(".pipe_status").join(format!("{}{}",stage.label.as_str(),n1.to_string().as_str())));
                        these_args.stage = None;
                        //these_args.output_file =
                        let string_args = these_args.to_vec();
                        let this_exe = std::env::current_exe().unwrap();
                        //let this_exe = this_exe.file_name().unwrap().to_str().unwrap();
                        let mut cmd = Command::new(&this_exe);
                        cmd.args(string_args);
                        // launch recursive call
                        if !cmd.output().expect(&format!("failed to launch {}",this_exe.to_str().unwrap())).status.success() {
                            println!("tried to run: {:?}",cmd);
                            panic!("recursive call failed");
                        }
                        // collect output
                        let s = utils::read_to_string(&these_args.output_file.clone().unwrap(),"json");
                        stat = serde_json::from_str(s.as_str()).expect("cannot deserialize struct");

                        // delete file after read
                        std::fs::remove_file(&these_args.output_file.clone().unwrap().with_extension("json")).expect(&format!("cannot remove {:?}",these_args.output_file));

                        println!("statusP: {}",s.as_str());
                    }else {
                        println!("{} is not started, but its not a pipe",stage.label);
                    }
                }
                _=> {
                    println!("{} returned with status {}",self.label,serde_json::to_string_pretty(&stat).unwrap());
                }
            }

            //todo(add weighting to configs, and weight the progress value here)
            let stage_weight=1.0;

            //let n_stages = stages.len() as f32;
            //println!("n_stages  ={}",n_stages);

            //let mut normalized_progress =  (stat.progress * stage_weight);// / stages.len() as f32;
            //normalized_progress = normalized_progress/ n_stages;
            //println!("normalized_progress  ={}",normalized_progress.to_float());
            if reverse{
                pipe_progress = pipe_progress - 1.0 + stat.progress.to_float();

                match stat.progress {
                    StatusType::Complete => {
                        total_pipe_status.children.push(stat);
                        break
                    }
                    _=> {}
                }
            }else {
                total_pipe_status.progress = total_pipe_status.progress + (stat.progress * stage_weight) / stages.len() as f32;

            }
            total_pipe_status.children.push(stat);
        }
        if reverse {
            total_pipe_status.children.reverse();
            total_pipe_status.progress = StatusType::InProgress(pipe_progress);
        }
        total_pipe_status
    }
}