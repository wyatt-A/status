use std::collections::HashMap;
use std::path::{Path, PathBuf};
use regex::Regex;
use serde::{Serialize, Deserialize, Deserializer};


#[derive(Serialize,Deserialize,Debug)]
pub struct Pipe {
    // a unit of work with a defined point completion
    pub label:String,
    pub stages:Vec<Stage>,
    pub source:Option<Vec<String>>
}

impl Pipe {

    pub fn open(pipe_conf:&Path) -> Self {
        let string = utils::read_to_string(pipe_conf,"txt");
        toml::from_str(&string).expect("cannot deserialize pipe!")
    }

    pub fn gen_template(pipe_conf:&Path) {
        let p = Self {
            label: String::from("what_this_pipe_is_called"),
            stages: vec![
                Stage{
                    label: String::from("some_stage_name"),
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap()
                },
                Stage{
                    label: String::from("some_second_stage_name"),
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap()
                },
            ],
            source: None
        };
        let str = toml::to_string(&p).expect("cannot deserialize struct");
        utils::write_to_file(pipe_conf,"txt",&str);
    }

    pub fn status(&self) -> Status {
        Status::NotStarted
    }
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Stage {
    pub label:String,
    #[serde(with = "serde_regex")]
    pub completion_file_pattern:Regex,

}


// impl Stage {
//     pub fn regex(&self) -> Regex{
//         Regex::new(&self.completion_file_pattern).expect("invalid regex!")
//     }
// }

impl StatusCheck for Stage {
    fn status(&self,dir:&Path,required_matches:&Vec<String>) -> Status {

        let contents = std::fs::read_dir(dir).expect("cannot read dir");

        let mut included = vec![];



        for thing in contents {

            let tp = thing.unwrap();

            let file_name = tp.path().file_name().unwrap().to_string_lossy().into_owned();
            let file_path = tp.path().to_string_lossy().into_owned();


            //println!("{:?}",thing_string);

            if self.completion_file_pattern.is_match(&file_path) {
                included.push(file_name)
            }





            //self.completion_file_pattern.is_match(thing.unwrap());

        }


        included.sort();
        let glob = included.join("/");


        let mut count = 0;

        for rm in required_matches {
            if Regex::new(&format!("(^|/).*{}.*($|/)",rm)).unwrap().is_match(&glob){
                count = count + 1;
                println!("{}",rm);
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

#[derive(Debug)]
pub enum Status {
    Complete,
    InProgress(f32),
    NotStarted,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Specimen {
    id:String,
    base_runno:String,
    runnos:Vec<String>,
    pipe:Pipe
}

impl Specimen {

}

pub trait Checkpoint {
    // a way to define the status of a pipe
    fn status(&self) -> Status;
    // load_status_file()
    // run some sort of status checker
    // return status

    fn complete(&self) -> bool {
        match self.status() {
            Status::Complete => true,
            _=> false
        }
    }
}





pub const PIPE_REGISTRY_LOCATION:&str = "./pipe_reg.txt";

pub struct PipeRegistry {

}

impl PipeRegistry {
    pub fn load() -> HashMap<String,PathBuf> {
        //todo!()
        let mut items = HashMap::<String,PathBuf>::new();
        items.insert(String::from("co_reg"),PathBuf::from("./test_pipe.txt"));
        items
    }
}


pub trait StatusCheck {
    fn status(&self,dir:&Path,required_matches:&Vec<String>) -> Status;
}


impl Checkpoint for Pipe {
    fn status(&self) -> Status {
        todo!()
    }
}





pub enum StatusChecker {
    Discrete,
    //OneToMany(),
    ManyToOne,
    ManyToMany,
}


#[derive(clap::Parser,Debug)]
pub struct StatusArgs {

    pub specimen_id:String,

    pub last_pipe:String,
}