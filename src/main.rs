

/* CIVM specimen status checker draft */

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize, Deserializer};
use toml;
use clap::Parser;
use regex::{Captures, Regex};
use utils;

#[derive(Debug)]
pub enum Status {
    Complete,
    InProgress(f32),
    NotStarted,
}

#[derive(Serialize,Deserialize,Debug)]
struct Specimen {
    id:String,
    base_runno:String,
    runnos:Vec<String>,
    pipe:Pipe
}

impl Specimen {

}

trait Checkpoint {
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

#[derive(Serialize,Deserialize,Debug)]
struct Pipe {
    // a unit of work with a defined point completion
    label:String,
    stages:Vec<Stage>,
    source:Option<Vec<String>>
}

impl Pipe {

    pub fn open(pipe_conf:&Path) -> Self {
        let string = utils::read_to_string(pipe_conf,"txt");
        toml::from_str(&string).expect("cannot deserialize pipe!")

        // read to string
        // toml deserialize to struct

        // dummy for now
        // let p = Pipe{
        //     label: String::from("co_reg"),
        //     stages: vec![
        //         Stage{
        //             label: String::from("make_header"),
        //             completion_file_pattern:
        //                 Regex::new(&String::from("inputs/.*nhdr")).unwrap()
        //
        //         },
        //         Stage{
        //             label: String::from("ants_registration"),
        //             completion_file_pattern:
        //             Regex::new(&String::from("results/.*[Aa]ffine.(mat|txt)")).unwrap()
        //
        //         },
        //         Stage{
        //             label: String::from("apply_transform"),
        //             completion_file_pattern:
        //             Regex::new(&String::from("results/Reg_.*nhdr")).unwrap()
        //
        //         }
        //     ],
        //     source: None
        // };

    }

    pub fn status(&self) -> Status {
        Status::NotStarted
    }
}


pub const PIPE_REGISTRY_LOCATION:&str = "./pipe_reg.txt";

struct PipeRegistry {

}

impl PipeRegistry {
    fn load() -> HashMap<String,PathBuf> {
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


#[derive(Serialize,Deserialize,Debug)]
struct Stage {
    label:String,
    #[serde(with = "serde_regex")]
    completion_file_pattern:Regex,

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


enum StatusChecker {
    Discrete,
    //OneToMany(),
    ManyToOne,
    ManyToMany,
}


#[derive(clap::Parser,Debug)]
struct StatusArgs {

    pub specimen_id:String,

    pub last_pipe:String,
}

fn main() {
    let args = StatusArgs::parse();
    // specimen
    println!("Status Check");
    println!("{:?}",args);

    let pipes = PipeRegistry::load();


    let pipe_conf = match pipes.get(&args.last_pipe) {
        None => {
            println!("available pipes: {:?}",pipes);
            panic!("cannot find specified pipe {}",args.last_pipe);
        }
        Some(pipe_conf) => pipe_conf
    };

    let p = Pipe::open(&pipe_conf);



    // Look for specimen registration file (this should happen at scan time)
    // let spec = Specimen {
    //     id: args.specimen_id.clone(),
    //     base_runno: String::from("N12345"),
    //     pipe: p.,
    //     runnos: vec![
    //
    //     ]
    // };


    let dummy_dir = vec![
        String::from("N60197_m00"),
        String::from(("N60197_m01")),
    ];

    // forward checking of stages
    for stage in &p.stages {
        let stage_stat = stage.status(Path::new("/Users/Wyatt/scratch/co_reg_N60197_m00-inputs"),&dummy_dir);
        println!("{}",stage.label);
        println!("{:?}",stage_stat);
    }



}



#[test]
fn test(){

    let test_string = "sdfajhsdjf";

    let re = Regex::new(r"(d).*(j)").expect("invalid regex");

    let result = re.replace(test_string, |caps: &Captures| {
        format!("aaa {}",&caps[2])
        //caps[1].to_owned()
    });


    println!("{:?}",result);

}
