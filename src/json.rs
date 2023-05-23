use serde::{Serialize, Deserialize};
use clap::Parser;

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