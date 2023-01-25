

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
use status::pipe::{PipeStatus, PipeRegistry, StatusArgs, StatusCheck};



fn main() {
    let args = StatusArgs::parse();
    // specimen
    println!("Status Check");
    println!("{:?}",args);

    let registered_pipes = PipeRegistry::load(Path::new("/Users/Wyatt/IdeaProjects/status/pipe_registry"));

    println!("pipes = {:?}",registered_pipes);



    let pipe_status = match registered_pipes.get(&args.last_pipe) {
        None => {
            println!("available pipes: {:?}",registered_pipes);
            panic!("cannot find specified pipe {}",args.last_pipe);
        }
        Some(pipe_conf) => pipe_conf
    };



    let pipe_status_args = vec![
        String::from("N51016_m0"),
        String::from("N51016_m1"),
        String::from("N51016_m2"),
        String::from("N51016_m3"),
        String::from("N51016_m4"),
        String::from("N51016_m5"),
        String::from("N51016_m6"),
    ];

    //let stat = pipe_status.stages[0].status(&pipe_status_args);
    //todo( migrate this loop to a pipe status check prior to implementing stage=pipe lookup in pipe registry)
    let base_runno = String::from("N51016");
    //println!("{:?}",stat);
    //forward checking of stages


    pipe_status.status(&pipe_status_args,Some(base_runno.as_str()));

    // for stage in &pipe_status.stages {
    //     //todo(smartly pass base_runno when required)
    //     let stage_stat = stage.status(&pipe_status_args,Some(base_runno.as_str()));
    //     //todo(stop checking if no progress in stage)
    //     println!("{}",stage.label);
    //     println!("{:?}",stage_stat);
    // }



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
