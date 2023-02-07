/* CIVM specimen status checker draft */

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
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
use status::stage::Stage;

fn main() {
    let args:StatusArgs = StatusArgs::parse();

    // specimen
    //println!("Status Check");
    //println!("{:?}",args);


    // check that big disk args is correct
    args.biggus_diskus();

    let status_dir = std::env::home_dir().expect("home dir cannot be fetched. Is the function deprecated?").join(".pipe_status");

    if !status_dir.exists() {
        std::fs::create_dir(&status_dir).expect(&format!("cannot create {:?}",status_dir));
    }


    let registered_pipes = match &args.config_dir {
        Some(config_dir) => {
            PipeRegistry::load_dir(config_dir)
        }
        None => {
            let pipe_registry_file = args.clone().pipe_registry.unwrap_or(PathBuf::from("/Users/Wyatt/IdeaProjects/status/pipe_configs/pipe_registry"));
            PipeRegistry::load(&pipe_registry_file)
        }
    };

    //println!("pipes = {:?}",registered_pipes);

    let pipe_status_conf = match registered_pipes.get(&args.last_pipe) {
        Some(pipe_conf) => pipe_conf,
        None => {
            println!("available pipes: {:?}",registered_pipes);
            panic!("cannot find specified pipe {}",args.last_pipe);
        }
    };

    let mut pipe_status_conf = pipe_status_conf.clone();

    pipe_status_conf.set_registry(&registered_pipes);


    let base_runno = args.specimen_id.clone();

    // try to get a list file from big disk to help with runno expansion
    let big_disk = std::env::var("BIGGUS_DISKUS").expect("BIGGUS_DISKUS is not set");

    let list_file = Path::new(&big_disk).join(&base_runno).with_extension("list");

    let runno_list:Vec<String> = match list_file.exists(){
        true => {
            println!("LIST FILE FOUND!");
            let s = utils::read_to_string(&list_file,"list");
            let re = regex::Regex::new(r",|\s+").unwrap();
            re.split(s.as_str()).map(|s| s.to_string()).collect()
        }
        false => {
            vec![]
        }
    };


    println!("running status check for {} ...",pipe_status_conf.label);

    let stat = pipe_status_conf.status(&args,&runno_list,Some(base_runno));

    // write status to file if the output is defined
    match &args.output_file {
        Some(file) => {
            match std::fs::create_dir(file.parent().expect("file has not parent")) {
                Err(_) => {},
                Ok(_) => {}
            }
            let s = serde_json::to_string_pretty(&stat).expect("cannot serialize struct");
            utils::write_to_file(file,"json",&s);
        }
        _=> {}
    }
    println!("--------------------FINAL STATUS OUTPUT----------------------");
    println!("{}",serde_json::to_string_pretty(&stat).unwrap());
}

