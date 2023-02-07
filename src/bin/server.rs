use serde_json;
use status::client::Request;
use clap::Parser;


#[derive(clap::Parser,Debug,Clone)]
pub struct ServerArgs {
    pub request:String,
}

fn main(){
    println!("this is the server!");

    let args:ServerArgs = ServerArgs::parse();
    let request:Request = serde_json::from_str(&args.request).expect("problem loading request");

    let pipe = request.configs.get_pipe(&request.pipe);
    let stage = pipe.get_stage(&request.stage);

    let status = stage.file_check(&request.status_args,&request.required_matches,request.base_runno.clone());
    println!("||{}||",serde_json::to_string(&status).expect("unable to serialize status"));
}