/* CIVM specimen status checker draft */

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize, Deserializer};
use toml;
use clap::Parser;
use regex::{Captures, Regex};
use utils;
use status::args::StatusArgs;
use status::pipe_registry::PipeRegistry;
use status::status_check::StatusCheck;
use status::pipe_status;
use status::pipe_status::PipeStatusConfig;



fn main() {
    let args = StatusArgs::parse();

    // specimen
    println!("Status Check");
    println!("{:?}",args);

    let registered_pipes = PipeRegistry::load(Path::new("/Users/Wyatt/IdeaProjects/status/pipe_registry"));

    println!("pipes = {:?}",registered_pipes);

    let pipe_status_conf = match registered_pipes.get(&args.last_pipe) {
        Some(pipe_conf) => pipe_conf,
        None => {
            println!("available pipes: {:?}",registered_pipes);
            panic!("cannot find specified pipe {}",args.last_pipe);
        }
    };

    // update liset for S69478
    let runno_list:Vec<String> = "N51016_m0,N51016_m1,N51016_m2,N51016_m3,N51016_m4,N51016_m5,N51016_m6"
        .to_string()
        .split(",")
        .map(|str| str.to_string())
        .collect();

    //println!("{:?}",runno_list);

    //let stat = pipe_status.stages[0].status(&pipe_status_args);
    //todo decode specimen id and run number and volume runno listing.
    let base_runno = args.specimen_id.clone();
    //println!("{:?}",stat);
    //forward checking of stages
    pipe_status_conf.status(&args,&runno_list,Some(base_runno.as_str()));
    //     //todo(smartly pass base_runno when required)
    //     let stage_stat = stage.status(&pipe_status_args,Some(base_runno.as_str()));
    //     //todo(stop checking if no progress in stage)
    //     println!("{}",stage.label);
    //     println!("{:?}",stage_stat);
    // }
}