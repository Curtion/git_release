use serde::{Serialize, Deserialize};
use clap::Parser;

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

    // #[serde(rename = "job_creator")]
    // pub job_creator: String,

    // #[serde(rename = "user_name")]
    // pub user_name: String,

    // #[serde(rename = "last_build_time")]
    // pub last_build_time: i64,

    // #[serde(rename = "health_score")]
    // pub health_score: i64,

    // #[serde(rename = "source_code")]
    // pub source_code: String,

    // #[serde(rename = "last_build_status")]
    // pub last_build_status: String,

    // #[serde(rename = "is_finished")]
    // pub is_finished: bool,

    // #[serde(rename = "disabled")]
    // pub disabled: bool,

    // #[serde(rename = "favorite")]
    // pub favorite: bool,

    // #[serde(rename = "is_modify")]
    // pub is_modify: bool,

    // #[serde(rename = "is_delete")]
    // pub is_delete: bool,

    // #[serde(rename = "is_execute")]
    // pub is_execute: bool,

    // #[serde(rename = "is_copy")]
    // pub is_copy: bool,

    // #[serde(rename = "is_forbidden")]
    // pub is_forbidden: bool,

    // #[serde(rename = "is_view")]
    // pub is_view: bool,

    // #[serde(rename = "task_id")]
    // pub task_id: String,

    // #[serde(rename = "code_branch")]
    // pub code_branch: String,

    // #[serde(rename = "commit_id")]
    // pub commit_id: String,

    // #[serde(rename = "trigger_type")]
    // pub trigger_type: String,

    // #[serde(rename = "build_time")]
    // pub build_time: i64,

    // #[serde(rename = "scm_web_url")]
    // pub scm_web_url: String,

    // #[serde(rename = "scm_type")]
    // pub scm_type: String,

    // #[serde(rename = "repo_id")]
    // pub repo_id: String,

    // #[serde(rename = "commit_detail_url")]
    // pub commit_detail_url: String,

    // #[serde(rename = "build_number")]
    // pub build_number: String,

    // #[serde(rename = "forbidden_msg")]
    // pub forbidden_msg: String,

    // #[serde(rename = "build_project_id")]
    // pub build_project_id: String,
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "")]
    pub path: String,
}

pub struct Repository {
    pub paths: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Deploy {
    pub path: String,
    pub tag: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub huawei: Huawei,
    pub region: Region,
    pub url: Url,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Huawei {
    pub domain: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Region {
    pub project_id: String,
    pub project_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Url {
    pub iam: String,
    pub cloudbuild: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTOKEN {
    #[serde(rename = "auth")]
    pub auth: Auth,
}

#[derive(Serialize, Deserialize)]
pub struct Auth {
    #[serde(rename = "identity")]
    pub identity: Identity,

    #[serde(rename = "scope")]
    pub scope: Scope,
}

#[derive(Serialize, Deserialize)]
pub struct Identity {
    #[serde(rename = "methods")]
    pub methods: Vec<String>,

    #[serde(rename = "password")]
    pub password: Password,
}

#[derive(Serialize, Deserialize)]
pub struct Password {
    #[serde(rename = "user")]
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "domain")]
    pub domain: Domain,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "password")]
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Domain {
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Scope {
    #[serde(rename = "project")]
    pub project: Project,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}