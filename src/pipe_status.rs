use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize, Deserializer};
use pathsearch::find_executable_in_path;
use regex::{Captures, Regex};


#[derive(Serialize,Deserialize,Debug)]
pub struct PipeStatus {
    // a unit of work with a defined point completion
    pub label:String,
    pub stages:Vec<Stage>,
    pub source:Option<Vec<String>>
}

impl PipeStatus {

    pub fn open(pipe_conf:&Path) -> Self {
        let string = utils::read_to_string(pipe_conf,"toml");
        toml::from_str(&string).expect("cannot deserialize pipe!")
    }

    pub fn gen_template(pipe_conf:&Path) {
        let p = Self {
            label: String::from("what_this_pipe_is_called"),
            stages: vec![
                Stage{
                    label: String::from("some_stage_name"),
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap(),
                    directory_pattern: "co_reg_${RUNNO}-inputs".to_string(),
                    signature_type:SignatureType::ManyToMany,
                },
                Stage{
                    label: String::from("some_second_stage_name"),
                    completion_file_pattern:
                    Regex::new(&String::from("valid_regular_expression_file_pattern")).unwrap(),
                    directory_pattern: "co_reg_${RUNNO}-results".to_string(),
                    signature_type:SignatureType::ManyToMany,
                },
            ],
            source: None
        };
        let str = toml::to_string(&p).expect("cannot deserialize struct");
        utils::write_to_file(pipe_conf,"txt",&str);
    }

}



#[derive(Serialize,Deserialize,Debug)]
pub enum SignatureType {
    Discrete,
    //OneToMany(),
    ManyToOne,
    ManyToMany,
}



// impl Stage {
//     pub fn regex(&self) -> Regex{
//         Regex::new(&self.completion_file_pattern).expect("invalid regex!")
//     }
// }

impl StatusCheck for PipeStatus {
    fn status(&self, required_matches: &Vec<String>, base_runno: Option<&str>) -> Status {

        let mut n_complete:f32 = 0.0;
        for stage in &self.stages {
            //todo(smartly pass base_runno when required)
            let stage_stat = stage.status(&required_matches,base_runno);
            //todo(stop checking if no progress in stage)
            println!("{}",stage.label);
            println!("{:?}",stage_stat);

            match &stage_stat {
                Status::Complete => n_complete = n_complete + 1.0,
                Status::InProgress(_) | Status::NotStarted => {
                }

            }

            let stat = match &stage_stat {
                Status::InProgress(_) | Status::NotStarted => {
                    //check the pipe table
                    let registered_pipes = PipeRegistry::load(Path::new("/Users/Wyatt/IdeaProjects/status/pipe_registry"));
                    match registered_pipes.get(&self.label) {
                        None => {
                            Status::NotStarted
                        }
                        Some(pipe_conf) => {
                            pipe_conf.status(&required_matches, base_runno)
                        }
                    }
                }
                _ => Status::Complete
            };

            match &stat {
                Status::Complete => n_complete = n_complete + 1.0,
                Status::InProgress(prog) => n_complete = n_complete + prog,
                _=> {}
            }



        }

        //Status::InProgress(n_complete/self.stages.len() as f32)

        Status::NotStarted

    }
}


impl StatusCheck for Stage {
    fn status(&self,required_matches:&Vec<String>,base_runno:Option<&str>) -> Status {
        use SignatureType::*;

        //self.directory_pattern

        // match literall ${} characters with only word characters including ${} in match
        let re = Regex::new(r"(\$\{[[:alnum:]_]+\})").unwrap();
        let big_disk = std::env::var("BIGGUS_DISKUS").expect("BIGGUS_DISKUS is not set");
        let mut the_dir = self.directory_pattern.clone();
        re.captures_iter(&self.directory_pattern).for_each(|captures|{
            for cap_idx in 1..captures.len(){
                if captures[cap_idx].eq("${BIGGUS_DISKUS}") {
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&big_disk);
                }else if captures[cap_idx].eq("${PARAM0}") {
                    //todo(deal with 0 match?)
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&required_matches[0]);
                }else if captures[cap_idx].eq("${PREFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"diffusion");
                }
                else if captures[cap_idx].eq("${SUFFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }
                else if captures[cap_idx].eq("${PROGRAM}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"dsi_studio");
                }
                else if captures[cap_idx].eq("${BASE}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&base_runno.clone().unwrap_or("BASE_RUNNO"));
                }
                else if captures[cap_idx].eq("${SEP}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }else {
                    panic!("capture not recognized")
                }
            }
        });

        // directory should be valid now... if its missing we cant check?... or this stage has 0 progress
        // return not started?


        match &self.signature_type {
            ManyToMany => {

            }
            _=> {
                panic!("signature type not yet implemented")
            }

        }

        let contents = std::fs::read_dir(&the_dir).expect(&format!("{} doesn't exist",the_dir));
        let mut included = vec![];
        for thing in contents {
            let tp = thing.unwrap();
            let file_name = tp.path().file_name().unwrap().to_string_lossy().into_owned();
            let file_path = tp.path().to_string_lossy().into_owned();
            if self.completion_file_pattern.is_match(&file_path) {
                included.push(file_name)
            }
        }


        included.sort();
        let glob = included.join("/");

        let mut count = 0;

        for rm in required_matches {
            if Regex::new(&format!("(^|/).*{}.*($|/)",rm)).unwrap().is_match(&glob){
                count = count + 1;
                //println!("{}",rm);
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
    pipe: PipeStatus
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






#[derive(Serialize,Deserialize,Debug)]
pub struct PipeRegistry {
    items:Vec<PathBuf>
}




//todo!(a pipe is proagram that runs from the command line pipe registry just hold the alias for the pipe. Program paths get resolved on load)


impl PipeRegistry {
    pub fn load(pipe_registery:&Path) -> HashMap<String,PipeStatus> {
        let txt = utils::read_to_string(pipe_registery,"txt");
        let reg:PipeRegistry = toml::from_str(&txt).expect("unable to deserialuze. File is corrupt");

        println!("{:?}",reg);
        reg.resolve()
    }


    pub fn resolve(&self) -> HashMap<String,PipeStatus> {
        let mut resolved_status = HashMap::<String,PipeStatus>::new();
        for exec in &self.items {
            if let Some(exe) = find_executable_in_path(exec) {
                let resolved_exe = std::fs::canonicalize(&exe).unwrap();
                let exec_filename = exe.file_name().expect("exe has no file name");
                let exe_dir = resolved_exe.parent().expect("exe has no parent dir");
                let pipe_config_filename = format!("{}_status",exec_filename.to_str().unwrap());
                let ps = PipeStatus::open(&exe_dir.join(pipe_config_filename).with_extension("toml"));
                resolved_status.insert(ps.label.clone(),ps);
            }
        }
        resolved_status
    }
}


pub trait StatusCheck {
    fn status(&self,required_matches:&Vec<String>,base_runno:Option<&str>) -> Status;
}





#[derive(clap::Parser,Debug)]
pub struct StatusArgs {
    pub specimen_id:String,
    pub last_pipe:String,
}