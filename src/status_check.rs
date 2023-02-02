use crate::args::StatusArgs;


// pub enum Status {
//     Complete,
//     InProgress(f32),
//     NotStarted,
// }

#[derive(Debug)]
pub enum StatusType {
    InProgress(f32),
    NotStarted,
    Complete,
}

#[derive(Debug)]
pub struct Status {
    pub label:String,
    pub progress:StatusType,
    pub children:Vec<Status>
}

pub trait StatusCheck {
    fn status(&self,user_args:&StatusArgs ,required_matches: &Vec<String>, base_runno: Option<&str>) -> Status;
}