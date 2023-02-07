use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
// gather list of pipe status files and package them as json for shipping
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::pipe_status::PipeStatusConfig;
use serde::{Serialize, Deserialize};
use ssh_rs::{LocalSession, LocalShell, SessionConnector, ssh, SshErrorKind, SshResult};
use ssh_config::SSHConfig;
use crate::args::StatusArgs;
use crate::remote_system::SshError;


#[derive(Serialize,Deserialize,Debug)]
pub struct ConfigCollection {
    configs:HashMap<String,PipeStatusConfig>
}

impl ConfigCollection {

    pub fn from_dir(dir:&Path) -> Self {
        let mut configs = HashMap::<String,PipeStatusConfig>::new();
            match utils::find_files(dir,"toml",true) {
            Some(files) => {
                for file in files {
                    let toml_str = utils::read_to_string(&file,"toml");
                    let cfg:PipeStatusConfig = toml::from_str(&toml_str).expect("unable to load config!");
                    configs.insert(cfg.label.clone(),cfg);
                }
                ConfigCollection{configs}
            },
            None => panic!("no config files found!")
        }
    }

    pub fn servers(&self) -> HashSet<String> {

        let mut servers = HashSet::<String>::new();

        for (_,cfg) in &self.configs {
            match &cfg.preferred_computer {
                Some(computers) => {
                    for computer in computers {
                        servers.insert(computer.clone());
                    }
                }
                None => {}
            }
            for stage in &cfg.stages {
                match &stage.preferred_computer {
                    Some(computers) => {
                        for computer in computers {
                            servers.insert(computer.clone());
                        }
                    }
                    None => {}
                }
            }
        }
        servers
    }

    pub fn get_pipe(&self,pipe_name:&str) -> &PipeStatusConfig {
        self.configs.get(pipe_name).expect(&format!("{} isn't defined in pipe_configs",pipe_name))
    }

}



#[derive(Debug)]
pub enum ClientError {
    ConfigLoadError,
    ConfigDirEmpty,
    PipeDoesntExist,
    ServerNotFound,
}



#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserArgs {
    run_number:String,
    pipeline:String,
}



pub struct Server {
    hostname:String,
    user_name:String,
    port:u32,
    session:LocalSession<TcpStream>,
    shell:LocalShell<TcpStream>
}

#[derive(Debug)]
pub enum ConnectionError {
    UnableToConnect,
    UnableToStartShell,
}

impl Server {
    pub fn new(hostname:&str, user_name:&str, port:u32) -> Result<Self,ConnectionError> {

        let private_key = std::env::home_dir().expect("cannot get home dir").join(".ssh").join("id_rsa");
        let mut session = ssh::create_session()
            .username(user_name)
            .private_key_path(private_key)
            .connect(&format!("{}:{}", hostname, port)).map_err(|_|ConnectionError::UnableToConnect)?
            .run_local();
        let mut shell = session.open_shell().map_err(|_|ConnectionError::UnableToStartShell)?;

        Ok(Server{
            hostname: hostname.to_string(),
            user_name: user_name.to_string(),
            port,
            session,
            shell,
        })
    }


    pub fn send_request(&mut self,request:&Request) {
        let req_string = serde_json::to_string(request).expect("unable to serialize request");
        let command_string = format!("status server --request={}\n",req_string);
        self.shell.write(command_string.as_bytes()).expect(&format!("unable to write to shell on {}",self.hostname));
    }

}



#[derive(Serialize,Deserialize,Debug)]
pub struct Request {
    pub configs:ConfigCollection,
    pub pipe:String,
    pub stage:String,
    pub status_args:StatusArgs,
    pub required_matches:Vec<String>,
    pub base_runno:Option<String>,
}

#[test]
fn test(){

    // read pipe configs directory and build the config collection
    let p = Path::new("./pipe_configs");

    let files = utils::find_files(&p,"toml",true).expect(&format!("no config files found in {:?}",p));


    let config_collection = ConfigCollection::from_dir(&p);

    let server_names = config_collection.servers();

    // client needs to open up connections to servers

    println!("servers = {:?}", server_names);


    let home = std::env::home_dir().expect("unable to resolve home directory");

    let ssh_config_file = home.join(".ssh").join("config");

    let config_str = utils::read_to_string(&ssh_config_file,"");

    let c = SSHConfig::parse_str(&config_str).unwrap();

    // resolve user for servers


    let mut known_servers = HashMap::<String,Server>::new();

    for server_name in &server_names {
        let server_info = c.query(server_name);
        if server_info.is_empty(){
            println!("no ssh config found for {} in {:?}", server_name, ssh_config_file);
        }
        else {
            match server_info.get("User") {
                Some(user) => {
                    match Server::new(server_name, user, 22) {
                        Ok(server) =>{
                            println!("connection to {} successful ...",server_name);
                            known_servers.insert(server_name.to_string(),server);
                        }
                        Err(e) => {
                            println!("problem occurred with {} connection with error: {:?}",server_name,e)
                        }
                    }
                }
                None => {
                    println!("no user specified for {}", server_name);
                }
            }
        }
    }

    let p = config_collection.get_pipe("co_reg");

    // for each stage, get the list of preferred computers
    // loop over each computer and request a stage status.
    // if the stage status returns incomplete, check if it is also another pipe,
    // if it is, run the pipe status

    // for stage in &p.stages {
    //
    // }




}

