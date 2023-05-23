use serde::{Serialize, Deserialize};

// 任务列表响应JSON
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectList {
    #[serde(rename = "total")]
    pub total: i64,

    #[serde(rename = "jobs")]
    pub jobs: Vec<Job>,
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "job_name")]
    pub job_name: String,
}

// 构建任务请求JSON
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildJob {
    #[serde(rename = "job_id")]
    pub job_id: String,
    pub scm: Scm,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scm {
    #[serde(rename = "build_tag")]
    pub build_tag: String,
}

// 构建任务状态响应JSON
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobStatus {
    #[serde(rename = "result")]
    pub result: bool,
}

// 任务响应JSON
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobDetail {
    #[serde(rename = "octopus_job_name")]
    pub octopus_job_name: String,
    #[serde(rename = "actual_build_number")]
    pub actual_build_number: String,
    #[serde(rename = "daily_build_number")]
    pub daily_build_number: String,
}

// 任务结果响应JSON
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobResult {
    #[serde(rename = "job_name")]
    pub job_name: String,
    #[serde(rename = "build_number")]
    pub build_number: i64,
    #[serde(rename = "project_id")]
    pub project_id: String,
    #[serde(rename = "project_name")]
    pub project_name: String,
    pub parameters: Parameters,
    #[serde(rename = "build_steps")]
    pub build_steps: Vec<BuildStep>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub code_branch: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildStep {
    pub name: String,
    pub status: String,
    #[serde(rename = "build_time")]
    pub build_time: i64,
}