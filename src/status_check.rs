use crate::args::StatusArgs;

#[derive(Debug)]
pub enum Status {
    Complete,
    InProgress(f32),
    NotStarted,
}

pub trait StatusCheck {
    fn status(&self,user_args:&StatusArgs ,required_matches: &Vec<String>, base_runno: Option<&str>) -> Status;
}