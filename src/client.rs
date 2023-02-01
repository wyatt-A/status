use std::cell::RefCell;
use std::collections::HashMap;
// gather list of pipe status files and package them as json for shipping
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::pipe_status::PipeStatusConfig;
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize,Debug)]
pub struct ConfigCollection {
    configs:Vec<PipeStatusConfig>
}

#[derive(Debug)]
pub enum ClientError {
    ConfigLoadError,
    ConfigDirEmpty,
    PipeDoesntExist,
    ServerNotFound,
}



#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Server {
    hostname:String,
    username:String,
    pubkey:PathBuf,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct ServerList {
    servers:Vec<Server>
}

impl ServerList {
    pub fn to_hash(&self) -> HashMap<String,Server> {
        let mut h = HashMap::<String,Server>::new();
        self.servers.iter().for_each(|s|{
            h.insert(s.hostname.clone(),s.clone());
        });
        h
    }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserArgs {
    run_number:String,
    pipeline:String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Request {
    user_args:UserArgs,
    config_collection:Vec<PipeStatusConfig>
}




pub fn build_server_list(user_args:&UserArgs, configs:&HashMap::<String, PipeStatusConfig>) -> Result<Vec<Server>,ClientError> {
    let computers = match configs.get(user_args.pipeline.as_str()) {
        Some(pipe) => {
            Ok(pipe.preferred_computer.clone().unwrap_or(vec![]))
        }
        None => Err(ClientError::PipeDoesntExist)
    }?;

    // convert list of computers to server objects
    let server_list_file = Path::new("/Users/Wyatt/IdeaProjects/status/server_list");
    let s = utils::read_to_string(&server_list_file,"toml");
    println!("{}",s);
    let servers:ServerList = toml::from_str(&s).expect(&format!("unable to parse {:?} to toml",server_list_file));
    let server_hash = servers.to_hash();
    let servers:Vec<Server> = computers.iter().map(|c|{
        match c.as_str() {
            _=> {
                server_hash.get(c.as_str()).ok_or(ClientError::ServerNotFound)?.clone()
            }
        }
    }).collect();
    Ok(servers)
}




/*
    Build a request file that will be sent to a computer that will run the server-side process
*/
pub fn build_request(user_args:&UserArgs, config_dir:&Path, request_file:&Path) -> Result<Vec<Server>,ClientError> {
    match utils::find_files(config_dir,"toml",true){
        Some(files) => {

            // build a hash for easy lookup

            let mut configs = HashMap::<String, PipeStatusConfig>::new();
            files.iter().for_each(|f|{
                let cfg_str = utils::read_to_string(f,"toml");
                let p: PipeStatusConfig = toml::from_str(cfg_str.as_str()).expect(&format!("cannot deserialize {:?}", f));
                configs.insert(p.label.clone(),p);
            });

            // get list of preferred computers for the pipe of interest

            let server_list = build_server_list(user_args,&configs)?;

            let configs:Vec<PipeStatusConfig> = files.iter().map(|f|{
                let cfg_str = utils::read_to_string(f,"toml");
                toml::from_str(cfg_str.as_str()).expect(&format!("cannot deserialize {:?}",f))
            }).collect();

            // find the server we need to run this on
            // check if the server actually local

            utils::write_to_file(
                request_file,
                "json",
                serde_json::to_string(
                    &Request{
                        user_args:user_args.clone(),
                        config_collection:configs
                    }
                ).map_err(|_|ClientError::ConfigLoadError)?.as_str()
            );
        Ok(server_list)
        }
        None => Err(ClientError::ConfigDirEmpty)
    }
}








#[test]
fn test(){


    let user_args = UserArgs{
        run_number:String::from("N60279"),
        pipeline:String::from("diffusion_calc_nlsam"),
    };

    // read pipe configs directory and build the config collection
    let p = Path::new("/Users/Wyatt/IdeaProjects/status/pipe_configs");
    let d = Path::new("/Users/Wyatt/IdeaProjects/status/work/pipe_cfg");

    let servers = build_request(&user_args,&p,&d).expect("trouble writing json");

    println!("{:?}",servers);
}