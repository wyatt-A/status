use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(clap::Parser,Debug,Clone,Serialize,Deserialize)]
pub struct StatusArgs {
    pub specimen_id:String,
    pub last_pipe:String,
    #[clap(short, long)]
    pub stage:Option<String>,
    #[clap(short, long)]
    pub config_dir:Option<PathBuf>,
    #[clap(short, long)]
    pub output_file:Option<PathBuf>,
    #[clap(short, long)]
    pub pipe_registry:Option<PathBuf>,
    #[clap(long)]
    pub BIGGUS_DISKUS:Option<String>,
    #[clap(short,long)]
    pub forward_check:Option<bool>,
}


impl StatusArgs {

    pub fn biggus_diskus(&self) -> Option<(String,String)> {
        match &self.BIGGUS_DISKUS {
            Some(arg) => {
                let arg = arg.to_string();
                let split:Vec<&str> = arg.split(":").collect();
                if split.len()  != 2 {
                    panic!("BIGGUS_DISKUS must contain a : for")
                }
                Some((
                    split[0].to_string(),
                    split[1].to_string()
                    ))
            }
            None => None
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} {}{}{}{}",
            self.specimen_id,
            self.last_pipe,
            match &self.stage {
                Some(stage) => format!(" --stage={}",stage),
                None => String::from("")
            },
            match &self.config_dir {
                Some(config_dir) => format!(" --config-dir={:?}",config_dir),
                None => String::from("")
            },
            match &self.output_file {
                Some(output_file) => format!(" --output-file={:?}",output_file),
                None => String::from("")
            }
        )
    }


    pub fn to_vec(&self) -> Vec<String> {

        let mut out = vec![
            self.specimen_id.clone(),
            self.last_pipe.clone(),
        ];

            match &self.stage {
                Some(stage) => out.push(format!("--stage={}",stage)),
                None => {  }
            }
            match &self.config_dir {
                Some(config_dir) => out.push(format!("--config-dir={}",config_dir.to_str().unwrap())),
                None => {  }
            }
            match &self.output_file {
                Some(output_file) =>{
                    out.push(format!("--output-file={}",output_file.to_str().unwrap()))
                } ,
                None => {  }
            }
        match &self.pipe_registry {
            Some(pipe_registry) =>{
                out.push(format!("--pipe-registry={}",pipe_registry.to_str().unwrap()))
            } ,
            None => {  }
        }
        match &self.BIGGUS_DISKUS {
            Some(BIGGUS_DISKUS) =>{
                out.push(format!("--biggus-diskus={}",BIGGUS_DISKUS))
            } ,
            None => {  }
        }
        out
    }
}