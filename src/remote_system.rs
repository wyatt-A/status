use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;
use ssh_rs::{LocalSession, SessionConnector, ssh};



#[derive(Clone)]
pub struct RemoteSystem {
    host_name:String,
    user_name:String,
    local_pub_key:PathBuf,
    pub port:u16,
}

#[derive(Debug)]
pub enum SshError {
    UnableToConnect,
    UnableToSendCommand,
    UnableToReadRemoteDir,
}


pub struct RemoteConnection {
    remote_system:RemoteSystem,
    connection:LocalSession<TcpStream>,
}

impl RemoteConnection {
    pub fn read_dir(&mut self,dir:&Path) -> Result<String,SshError> {
        let path_str = dir.to_owned().into_os_string().into_string().unwrap();
        let exec = self.connection.open_exec().map_err(|_|SshError::UnableToConnect)?;
        //Ok(String::from_utf8(exec.send_command(&format!("ls {}",path_str)).map_err(|_|SshError::UnableToSendCommand)?).unwrap())
        Ok(String::from_utf8(exec.send_command("ls").map_err(|_|SshError::UnableToSendCommand)?).unwrap())
    }

    pub fn send_command(&mut self,command:&str) -> String {
        let ex = self.connection.open_exec().unwrap();
        String::from_utf8(ex.send_command(command).unwrap()).unwrap()
    }
}



impl RemoteSystem {

    pub fn new(host_name:&str,user_name:&str,local_pub_key:&Path) -> Self {
        Self {
            host_name:host_name.to_string(),
            user_name:user_name.to_string(),
            local_pub_key:local_pub_key.to_owned(),
            port:22,
        }
    }

    pub fn connect(&self) -> Result<RemoteConnection,SshError> {

        let mut session = ssh::create_session()
        .username(&self.user_name)
        .private_key_path(&self.local_pub_key)
        .connect(&format!("{}:{}",self.host_name,self.port)).map_err(|_|SshError::UnableToConnect)?.run_local();
        Ok(
            RemoteConnection{
                remote_system: self.clone(),
                connection: session,
            }
        )
    }

}


#[test]
fn ssh_test() {

    ssh::enable_log();


    let remote_sys2 = RemoteSystem::new("civmcluster1","wa41",Path::new("/Users/Wyatt/.ssh/id_rsa"));
    let mut connection2 = remote_sys2.connect().unwrap();



    let response = connection2.send_command("ls -la");
    print!("response: {}",response);




    let mut cmd = std::process::Command::new("ssh");
    cmd.args(vec![
        "wa41@civmcluster1",
        r"echo \$BIGGUS_DISKUS"
    ]);

    let out = cmd.output().unwrap();

    

    println!("{:?}",out.status.success())

}

