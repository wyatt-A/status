//todo!(a pipe is proagram that runs from the command line pipe registry just hold the alias for the pipe. Program paths get resolved on load)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use pathsearch::find_executable_in_path;
use crate::pipe_status::PipeStatusConfig;
use serde::{Serialize, Deserialize};

pub const PIPE_REGISTRY_FILE:&str = "/Users/Wyatt/IdeaProjects/status/pipe_registry";

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipeRegistry {
    items:Vec<PathBuf>
}

impl PipeRegistry {
    pub fn load(pipe_registery:&Path) -> HashMap<String, PipeStatusConfig> {
        let txt = utils::read_to_string(pipe_registery,"toml");
        let reg:PipeRegistry = toml::from_str(&txt).expect("unable to deserialuze. File is corrupt");

        println!("{:?}",reg);
        reg.resolve()
    }

    pub fn resolve(&self) -> HashMap<String, PipeStatusConfig> {
        let mut resolved_status = HashMap::<String, PipeStatusConfig>::new();
        for exec in &self.items {

            let exec_path = Path::new(exec);
            match exec_path.is_file(){
                true => {
                    let ps = PipeStatusConfig::open(&exec_path.with_extension("toml"));
                    resolved_status.insert(ps.label.clone(),ps);
                }
                false => {
                    if let Some(exe) = find_executable_in_path(exec) {
                        let resolved_exe = std::fs::canonicalize(&exe).unwrap();
                        let exec_filename = exe.file_name().expect("exe has no file name");
                        let exe_dir = resolved_exe.parent().expect("exe has no parent dir");
                        let pipe_config_filename = format!("{}_status",exec_filename.to_str().unwrap());
                        let ps = PipeStatusConfig::open(&exe_dir.join(pipe_config_filename).with_extension("toml"));
                        resolved_status.insert(ps.label.clone(),ps);
                    }
                }
            }
        }
        resolved_status
    }
}