use std::ops;
use crate::args::StatusArgs;
use serde::{Serialize, Deserialize};
use crate::status_check::StatusType::NotStarted;

// pub enum Status {
//     Complete,
//     InProgress(f32),
//     NotStarted,
// }

#[derive(Debug,Serialize,Deserialize,Clone,Copy)]
pub enum StatusType {
    InProgress(f32),
    NotStarted,
    Complete,
    Invalid,
}

impl StatusType {
    pub fn to_float(&self) -> f32 {
        match &self {
            StatusType::Invalid => panic!("invalid status detected!"),
            StatusType::Complete => 1.0,
            StatusType::NotStarted => 0.0,
            StatusType::InProgress(prog) => *prog
        }
    }
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Status {
    pub label:String,
    pub progress:StatusType,
    pub children:Vec<Status>
}

impl ops::Add<f32> for StatusType {
    type Output = StatusType;
    fn add(self, _rhs: f32) -> StatusType {
        match &self {
            StatusType::Invalid => StatusType::Invalid,
            StatusType::Complete => StatusType::Complete,
            StatusType::NotStarted => StatusType::InProgress(_rhs),
            StatusType::InProgress(progress) => {
                let p = progress + _rhs;
                if p == 1.0 {
                    StatusType::Complete
                }else if p > 1.0 {
                    StatusType::Invalid
                }else {
                    StatusType::InProgress(p)
                }
            }
        }
    }
}

impl ops::Add<StatusType> for StatusType {
    type Output = StatusType;
    fn add(self, _rhs: StatusType) -> StatusType {
        use StatusType::*;



        match &self {
            StatusType::Invalid => StatusType::Invalid,
            StatusType::Complete => StatusType::Complete,
            StatusType::NotStarted =>{
                match _rhs {
                    StatusType::NotStarted => NotStarted,
                    _=> StatusType::InProgress(_rhs.to_float()),
                }
            },
            StatusType::InProgress(progress) => {
                let p = progress + _rhs.to_float();
                if p == 1.0 {
                    StatusType::Complete
                }else if p > 1.0 {
                    //StatusType::Invalid
                    println!("progress > 1.0: {}",p);
                    StatusType::InProgress(p)
                }else {
                    StatusType::InProgress(p)
                }
            }
        }
    }
}

impl ops::Mul<f32> for StatusType {
    type Output = StatusType;
    fn mul(self, _rhs: f32) -> StatusType {
        match &self {
            StatusType::Invalid => StatusType::Invalid,
            StatusType::Complete => StatusType::Complete,
            StatusType::NotStarted => StatusType::NotStarted,
            StatusType::InProgress(progress) => {
                let p = progress * _rhs;
                if p == 1.0 {
                    StatusType::Complete
                }else if p > 1.0 {
                    //StatusType::Invalid
                    println!("progress > 1.0: {}",p);
                    StatusType::InProgress(p)
                }
                else {
                    StatusType::InProgress(p)
                }
            }
        }
    }
}

impl ops::Div<f32> for StatusType {
    type Output = StatusType;
    fn div(self, _rhs: f32) -> StatusType {
        match &self {
            StatusType::Invalid => StatusType::Invalid,
            StatusType::NotStarted => StatusType::NotStarted,
            _ => {
                let progress = self.to_float();
                let p = progress / _rhs;
                if p == 1.0 {
                    StatusType::Complete
                }else if p > 1.0 {
                    //StatusType::Invalid
                    println!("progress > 1.0: {}",p);
                    StatusType::InProgress(p)
                }
                else {
                    StatusType::InProgress(p)
                }
            }
        }
    }
}

pub trait StatusCheck {
    fn status(&self,user_args:&StatusArgs ,required_matches: &Vec<String>, base_runno: Option<&str>) -> Status;
}