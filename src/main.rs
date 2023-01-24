

/* CIVM specimen status checker draft */

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize, Deserializer};
use toml;
use clap::Parser;
use regex::{Captures, Regex};
use utils;

use status::pipe;
use status::pipe::{Pipe, PipeRegistry, StatusArgs, StatusCheck};


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
