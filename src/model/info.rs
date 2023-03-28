use serde::{Deserialize, Serialize};

use crate::{
    store::{Model, Proc, Task},
    ActError, ActResult, Workflow,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcInfo {
    pub pid: String,
    pub name: String,
    pub mid: String,
    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
    // pub vars: Vars,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskInfo {
    pub pid: String,
    pub tid: String,
    pub nid: String,
    pub state: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub ver: u32,
    pub size: u32,
    pub time: i64,
    pub model: String,
}

impl ModelInfo {
    pub fn workflow(&self) -> ActResult<Workflow> {
        let m = serde_yaml::from_str::<Workflow>(&self.model);
        match m {
            Ok(mut m) => {
                m.set_ver(self.ver);
                Ok(m)
            }
            Err(err) => Err(ActError::ConvertError(err.to_string())),
        }
    }
}

impl From<Model> for ModelInfo {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            ver: m.ver,
            size: m.size,
            time: m.time,

            model: m.model,
        }
    }
}

impl From<&Proc> for ProcInfo {
    fn from(p: &Proc) -> Self {
        let model = Workflow::from_str(&p.model).unwrap();
        Self {
            pid: p.pid.clone(),
            name: model.name,
            mid: model.id,
            state: p.state.clone().into(),
            start_time: p.start_time,
            end_time: p.end_time,
        }
    }
}

impl From<Task> for TaskInfo {
    fn from(t: Task) -> Self {
        Self {
            pid: t.pid,
            tid: t.tid,
            nid: t.nid,
            state: t.state.into(),
            start_time: t.start_time,
            end_time: t.end_time,
        }
    }
}