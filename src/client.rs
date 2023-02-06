use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
// gather list of pipe status files and package them as json for shipping
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::pipe_status::PipeStatusConfig;
use serde::{Serialize, Deserialize};
use ssh_rs::{LocalSession, LocalShell, ssh, SshErrorKind};
use ssh_config::SSHConfig;
use crate::remote_system::SshError;


#[derive(Serialize,Deserialize,Debug)]
pub struct ConfigCollection {
    configs:Vec<PipeStatusConfig>
}

impl ConfigCollection {

    pub fn from_dir(dir:&Path) -> Self {
        let mut configs = vec![];
            match utils::find_files(dir,"toml",true) {
            Some(files) => {
                for file in files {
                    let toml_str = utils::read_to_string(&file,"toml");
                    let cfg:PipeStatusConfig = toml::from_str(&toml_str).expect("unable to load config!");
                    configs.push(cfg);
                }
                ConfigCollection{configs}
            },
            None => panic!("no config files found!")
        }
    }

    pub fn servers(&self) -> HashSet<String> {

        let mut servers = HashSet::<String>::new();

        for cfg in &self.configs {
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

#[derive(Debug,Serialize,Deserialize)]
pub struct Request {
    user_args:UserArgs,
    config_collection:Vec<PipeStatusConfig>
}

pub struct Server {
    hostname:String,
    user_name:String,
    port:u32,
    local_private_key_path:Option<PathBuf>,

    session:Option<LocalSession<TcpStream>>,
    shell:Option<LocalShell<TcpStream>>
}


impl Server {
    pub fn new(name:&str,user:&str,port:u32) -> Self {
        Server{
            hostname:name.to_string(),
            user_name:user.to_string(),
            port,
            local_private_key_path:None,
            session: None,
            shell:None,
        }
    }

    pub fn connect(&mut self) -> Result<(),SshError>{

        let priv_key = std::env::home_dir().expect("cannot get home dir").join(".ssh").join("id_rsa");

        let session = self.session.get_or_insert_with(||{
            ssh::create_session()
                .username(&self.user_name)
                .private_key_path(priv_key)
                .connect(&format!("{}:{}",self.hostname,self.port))
                .unwrap()
                .run_local()
        });

        let shell = self.shell.get_or_insert_with(||{
            session.open_shell().expect("unable to connect")
        });
        println!("shell successfully started");

        Ok(())
    }

}



// pub fn build_server_list(user_args:&UserArgs, configs:&HashMap::<String, PipeStatusConfig>) -> Result<Vec<Server>,ClientError> {
//     let computers = match configs.get(user_args.pipeline.as_str()) {
//         Some(pipe) => {
//             Ok(pipe.preferred_computer.clone().unwrap_or(vec![]))
//         }
//         None => Err(ClientError::PipeDoesntExist)
//     }?;
//
//     // convert list of computers to server objects
//     let server_list_file = Path::new("/Users/Wyatt/IdeaProjects/status/server_list");
//     let s = utils::read_to_string(&server_list_file,"toml");
//     println!("{}",s);
//     let servers:ServerList = toml::from_str(&s).expect(&format!("unable to parse {:?} to toml",server_list_file));
//     let server_hash = servers.to_hash();
//     let servers:Vec<Server> = computers.iter().map(|c|{
//         match c.as_str() {
//             _=> {
//                 server_hash.get(c.as_str()).ok_or(ClientError::ServerNotFound)?.clone()
//             }
//         }
//     }).collect();
//     Ok(servers)
// }




/*
    Build a request file that will be sent to a computer that will run the server-side process
*/
// pub fn build_request(user_args:&UserArgs, config_dir:&Path, request_file:&Path) -> Result<Vec<Server>,ClientError> {
//     match utils::find_files(config_dir,"toml",true){
//         Some(files) => {
//
//             // build a hash for easy lookup
//
//             let mut configs = HashMap::<String, PipeStatusConfig>::new();
//             files.iter().for_each(|f|{
//                 let cfg_str = utils::read_to_string(f,"toml");
//                 let p: PipeStatusConfig = toml::from_str(cfg_str.as_str()).expect(&format!("cannot deserialize {:?}", f));
//                 configs.insert(p.label.clone(),p);
//             });
//
//             // get list of preferred computers for the pipe of interest
//
//             let server_list = build_server_list(user_args,&configs)?;
//
//             let configs:Vec<PipeStatusConfig> = files.iter().map(|f|{
//                 let cfg_str = utils::read_to_string(f,"toml");
//                 toml::from_str(cfg_str.as_str()).expect(&format!("cannot deserialize {:?}",f))
//             }).collect();
//
//             // find the server we need to run this on
//             // check if the server actually local
//
//             utils::write_to_file(
//                 request_file,
//                 "json",
//                 serde_json::to_string(
//                     &Request{
//                         user_args:user_args.clone(),
//                         config_collection:configs
//                     }
//                 ).map_err(|_|ClientError::ConfigLoadError)?.as_str()
//             );
//         Ok(server_list)
//         }
//         None => Err(ClientError::ConfigDirEmpty)
//     }
// }



#[test]
fn test(){

    // read pipe configs directory and build the config collection
    let p = Path::new("/Users/Wyatt/IdeaProjects/status/pipe_configs");

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
                    known_servers.insert(server_name.to_string(),Server::new(server_name, user, 22));
                }
                None => {
                    println!("no user specified for {}", server_name);
                }
            }
        }
    }

    // attempt to connect
    println!("attempting to connect to servers ...");
    for (hostname,server) in &mut known_servers {
        println!("connecting to {} ...",hostname);
        server.connect().unwrap();
    }


    let mut delos = known_servers.get("delos").unwrap();





    // let q = c.query("civmcluster1");
    //
    // println!("q is empty:{}",q.is_empty());
    //
    //
    // println!("{:?}",q);

}













