use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize};
use regex::{Regex};
use crate::args::StatusArgs;
//se crate::client::UserArgs;
use crate::pipe_registry::{PIPE_REGISTRY_FILE, PipeRegistry};
use crate::stage::{SignatureType, Stage};
use crate::status_check::{Status, StatusCheck};





#[derive(Serialize,Deserialize,Debug,Clone)]
struct StageStatus {
    val:f32,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
struct PipeStatus {
    stages:Vec<StageStatus>,
    val:f32,
}



#[derive(Serialize,Deserialize,Debug)]
pub struct PipeStatusConfig {
    // a unit of work with a defined point completion
    pub label:String,
    pub preferred_computer:Option<Vec<String>>,
    pub stages:Vec<Stage>,
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

        let mut n_complete:f32 = 0.0;
        for stage in &self.stages {
            //todo(smartly pass base_runno when required)

            //todo! ensure the local computer is first in preferred computers

            match &stage.preferred_computer {
                Some(computers) => {

                    let mut args = user_args.clone();
                    args.output_file = Some(PathBuf::from(r"\$HOME/.spec_status_config_tmp/PIPENAME"));
                    let remote_temp_dir = Path::new(r"\$HOME/.spec_status_config_tmp");
                    args.config_dir = Some(remote_temp_dir.to_owned());
                    args.stage = Some(stage.label.clone());


                    for computer in computers {

                        match &user_args.config_dir {
                            Some(conf_dir) => {
                                // todo!(use make temp to get a directory)

                                let mut cmd = Command::new("scp");
                                cmd.args(vec![
                                    "-pr",
                                    &format!("{:?}",conf_dir),
                                    computer.as_str(),
                                    &format!(":{:?}",remote_temp_dir)
                                ]);

                                if !cmd.output().expect("failed to launch scp").status.success() {
                                    panic!("scp failed");
                                }
                            }
                            None => {/*no op*/}
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
                        let local_status_file= Path::new(r"$HOME/.spec_tatus_config_tmp/incoming");
                        let mut cmd = Command::new("scp").args(vec![
                            "-p",
                            computer.as_str(),
                            &format!(":{:?}",args.config_dir),
                            &format!("{:?}",local_status_file)
                        ]);

                    }

                }
                None => {

                }
            }


            // println!("running stage status for {} ...",stage.label);
            // let stage_stat = stage.status(&required_matches,base_runno);
            // //todo(stop checking if no progress in stage)
            // println!("stage returned with status {:?}",stage_stat);
            //
            // match &stage_stat {
            //     Status::Complete => n_complete = n_complete + 1.0,
            //     Status::InProgress(_) | Status::NotStarted => {
            //     }
            //
            // }
            //
            // let stat = match &stage_stat {
            //     Status::InProgress(_) | Status::NotStarted => {
            //         //check the pipe table
            //         let registered_pipes = PipeRegistry::load(Path::new(PIPE_REGISTRY_FILE));
            //
            //         let pipe_conf = registered_pipes.get(&stage.label);
            //
            //         println!("child pipe config = {:?}",pipe_conf);
            //
            //         match pipe_conf {
            //             None => {
            //                 panic!("pipe not started");
            //                 Status::NotStarted;
            //             }
            //             Some(pipe_conf) => {
            //                 pipe_conf.status(&required_matches, base_runno)
            //             }
            //         }
            //     }
            //     _ => Status::Complete
            // };
            //
            // match &stat {
            //     Status::Complete => n_complete = n_complete + 1.0,
            //     Status::InProgress(prog) => n_complete = n_complete + prog,
            //     _=> {}
            // }
            //


        }

        //Status::InProgress(n_complete/self.stages.len() as f32)

        Status::NotStarted

    }
}