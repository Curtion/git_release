mod json;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use microkv::MicroKV;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::{env, fs, io, thread};

fn main() {
    let args = json::Args::parse();
    let mut path = args.path;
    if path == "" {
        let dir = env::current_dir().expect("获取当前目录失败");
        path = dir.to_str().unwrap().to_string();
    }
    let dirs = fs::read_dir(path).expect("读取目录失败");
    let mut latest = json::Repository {
        paths: Vec::new(),
        tags: Vec::new(),
    };
    let mut old = json::Repository {
        paths: Vec::new(),
        tags: Vec::new(),
    };
    for entry in dirs {
        if let Ok(entry) = entry {
            let path = entry.path();
            let is_dir = path.is_dir();
            let path = path.to_str().expect("解析目录失败");
            if is_dir {
                let is_git = is_git_repository(path);
                if is_git == "true" {
                    let tag = get_git_tag_latest(path);
                    let commit: String = get_git_commit_latest(path);
                    let tagcommit = get_git_tagcommit_latest(&tag, path);
                    if !tagcommit.starts_with(&commit) {
                        println!("路径: {}", path);
                        println!("最新tag: {}", tag);
                        println!("最新commit: {}", commit);
                        println!("最新tag对应的commit: {}", tagcommit);
                        println!("---------------------------------------------");
                        latest.paths.push(path.to_string());
                        latest.tags.push(tag.to_string());
                    } else {
                        old.paths.push(path.to_string());
                        old.tags.push(tag.to_string())
                    }
                }
            }
        }
    }
    if latest.paths.len() == 0 {
        println!("没有需要更新的项目!以下为当前最新tag:");
        println!("---------------------------------------------");
        for i in 0..old.paths.len() {
            println!("{} {}", old.paths[i], old.tags[i]);
        }
        return;
    }
    let mut input_type = String::new();
    println!("请输入要更新的版本类型!");
    println!("1. 大版本");
    println!("2. 小版本");
    println!("3. 修复版本");
    println!("4. 退出");
    io::stdin()
        .read_line(&mut input_type)
        .expect("输入解析错误!");
    let input_type = input_type.trim().parse::<i32>().expect("输入格式错误!");
    if input_type != 1 && input_type != 2 && input_type != 3 {
        return;
    }
    let mut deploy_list: Vec<json::Deploy> = Vec::new();
    for (index, path) in latest.paths.iter().enumerate() {
        let last_tag = latest
            .tags
            .get(index)
            .expect("未知错误")
            .split("V")
            .collect::<Vec<&str>>()[1];
        let version = version_add_one(input_type, last_tag);
        create_git_tag(&version, path);
        git_push_tag(path);
        deploy_list.push(json::Deploy {
            path: path.to_string(),
            tag: version.to_string(),
        });
    }
    let mut input_type = String::new();
    println!("是否启动华为云部署? y/N");
    io::stdin()
        .read_line(&mut input_type)
        .expect("输入解析错误!");
    let input_type = input_type.trim();
    if input_type == "y" {
        deploy_job(deploy_list);
    } else {
        println!("已取消华为云部署!");
    }
}

fn is_success(job_result: &json::JobResult) -> bool {
    job_result
        .build_steps
        .iter()
        .all(|item| item.status == "success")
}

fn is_error(job_result: &json::JobResult) -> bool {
    job_result
        .build_steps
        .iter()
        .any(|item| item.status == "error")
}

// 开始部署任务
fn deploy_job(deploy_list: Vec<json::Deploy>) {
    let config = parse_user_toml();
    let db = init_user_db();
    get_huawei_token(&db, &config);
    let token = db.get_unwrap::<String>("token").expect("获取本地token失败");
    let jobs = get_huawei_jobs(&db, &config).unwrap();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let m = MultiProgress::new();
    let handles: Vec<_> = deploy_list
        .into_iter()
        .map(|item| {
            let count: u64 = 36000;
            let pb = m.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());
            let job_name =  Path::new(&item.path).file_name().unwrap().to_str().unwrap();
            let job_id = jobs
                .jobs
                .iter()
                .find(|job| job.job_name == job_name.to_string())
                .and_then(|job| Some(job.id.clone()));
            let tag = "V".to_string() + &item.tag;
            let config = config.clone();
            let token = token.clone();
            pb.set_prefix(format!("[{}/{}]", job_name, tag));
            let job_name = job_name.to_string().clone();
            thread::spawn(move || -> Result<(), String> {
                let job_info = match job_id {
                    Some(ref job_id) => {
                        let job_info = huawei_run_job(&token, &config, &job_id, &tag).unwrap(); // 运行任务
                        let build_number = job_info.actual_build_number; // 获取任务构建number
                        Some((job_id, build_number))
                    }
                    None => {
                        pb.finish_with_message("未找到华为云任务");
                        return Ok(());
                    }
                }
                .unwrap();
                let job_id = job_info.0;
                let build_number = job_info.1;
                thread::sleep(Duration::from_millis(5000));
                for i in 0..count {
                    pb.inc(1);
                    if i % 200 == 0 {
                        pb.set_message(format!("{}/{}", i / 200, count / 200));
                        let job_result =
                            huawei_result_job(&token, &config, &job_id, &build_number).unwrap();
                        let is_success = is_success(&job_result);
                        let is_error = is_error(&job_result);
                        if is_success {
                            println!("{job_name} 任务构建成功!");
                            pb.finish_with_message("success");
                            return Ok(());
                        } else {
                            if is_error {
                                pb.finish_with_message("error");
                                println!("{job_name} 任务构建失败!以下为错误信息:");
                                for step in job_result.build_steps {
                                    if step.status == "error" {
                                        println!("{}: {}", step.name, step.status);
                                    }
                                }
                                return Err(String::from("error"));
                            }
                            continue;
                        }
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                pb.finish_with_message("Time Out");
                return Err(String::from("Time Out"));
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    m.clear().unwrap();
}

// 查询华为云任务列表
#[tokio::main]
async fn get_huawei_jobs(
    db: &MicroKV,
    config: &json::Config,
) -> Result<json::ProjectList, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .get(
            config.url.cloudbuild.clone()
                + "/v3/ec92bf3022ec42b3b04c30c73d81f23a/jobs?page_index=0&page_size=100",
        )
        .header(
            "X-Auth-Token",
            db.get_unwrap::<String>("token").expect("获取本地token失败"),
        )
        .send()
        .await?
        .json::<json::ProjectList>()
        .await?;
    Ok(res)
}

// 获取华为云TOKEN
fn get_huawei_token(db: &MicroKV, config: &json::Config) {
    match db.get_unwrap::<String>("token") {
        Ok(_) => match huawei_check_token(db, config) {
            Ok(_) => {
                println!("华为云已登录");
            }
            Err(_) => {
                println!("华为云登录过期,开始登录");
                huawei_login(db, config).unwrap();
            }
        },
        Err(_) => {
            println!("华为云未登录,开始登录");
            huawei_login(db, config).unwrap();
        }
    }
}

// 华为云登录
#[tokio::main]
async fn huawei_login(db: &MicroKV, config: &json::Config) -> Result<(), Box<dyn std::error::Error>> {
    let json = json::GetTOKEN {
        auth: json::Auth {
            identity: json::Identity {
                methods: vec!["password".to_string()],
                password: json::Password {
                    user: json::User {
                        name: config.huawei.name.clone(),
                        password: config.huawei.password.clone(),
                        domain: json::Domain {
                            name: config.huawei.domain.clone(),
                        },
                    },
                },
            },
            scope: json::Scope {
                project: json::Project {
                    name: config.region.project_name.clone(),
                    id: config.region.project_id.clone(),
                },
            },
        },
    };
    let client = reqwest::Client::new();
    let res = client
        .post(config.url.iam.clone() + "/v3/auth/tokens")
        .json(&json)
        .send()
        .await?;
    let token = res
        .headers()
        .get("x-subject-token")
        .expect("华为云登录失败")
        .to_str()
        .unwrap();
    db.put("token", &token).expect("缓存token失败");
    println!("华为云登录成功");
    Ok(())
}

// 华为云检测token是否过期
#[tokio::main]
async fn huawei_check_token(
    db: &MicroKV,
    config: &json::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .get(config.url.iam.clone() + "/v3/auth/tokens")
        .header(
            "X-Auth-Token",
            db.get_unwrap::<String>("token").expect("获取本地token失败"),
        )
        .header(
            "X-Subject-Token",
            db.get_unwrap::<String>("token").expect("获取本地token失败"),
        )
        .send()
        .await?;
    if res.status() == 200 {
        Ok(())
    } else {
        Err("华为云token过期,重新登录".into())
    }
}

// 华为云开始构建任务
#[tokio::main]
async fn huawei_run_job(
    token: &str,
    config: &json::Config,
    jobid: &str,
    tag: &str,
) -> Result<json::JobDetail, Box<dyn std::error::Error>> {
    let json = json::BuildJob {
        job_id: jobid.to_string(),
        scm: json::Scm {
            build_tag: tag.to_string(),
        },
    };
    let client = reqwest::Client::new();
    let res = client
        .post(config.url.cloudbuild.clone() + "/v3/jobs/build")
        .header("X-Auth-Token", token)
        .json(&json)
        .send()
        .await?;
    if res.status() == 200 {
        let resjson = res.json::<json::JobDetail>().await?;
        Ok(resjson)
    } else {
        Err(res.text().await?.into())
    }
}

// 华为云查看任务结果
#[tokio::main]
async fn huawei_result_job(
    token: &str,
    config: &json::Config,
    job_id: &str,
    build_number: &str,
) -> Result<json::JobResult, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client
        .get(
            config.url.cloudbuild.clone()
                + "/v3/jobs/"
                + job_id
                + "/"
                + build_number
                + "/history-details",
        )
        .header("X-Auth-Token", token)
        .send()
        .await?;
    if res.status() == 200 {
        Ok(res.json::<json::JobResult>().await?)
    } else {
        Err(res.text().await?.into())
    }
}

// 解析配置文件
fn parse_user_toml() -> json::Config {
    let exe_path = env::current_exe().expect("获取当前路径失败");
    let exe_path = exe_path.to_str().unwrap();
    let exe_dir = Path::new(exe_path).parent().unwrap();
    let toml_str = fs::read_to_string(exe_dir.join("user.toml")).expect("读取配置文件失败");
    let config: json::Config = toml::from_str(&toml_str).unwrap();
    return config;
}

// 初始持久化db
fn init_user_db() -> MicroKV {
    let unsafe_pwd: String = String::from("jidian@iot");
    let mut db = MicroKV::open("user_db").unwrap().with_pwd_clear(unsafe_pwd);
    db = db.set_auto_commit(true);
    return db;
}

// 推送tag
fn git_push_tag(path: &str) {
    let output = Command::new("git")
        .arg("push")
        .arg("--tags")
        .current_dir(path)
        .output()
        .expect("执行git push tag失败");
    if output.status.success() {
        // println!("推送tag成功!");
    } else {
        let msg = String::from_utf8(output.stdout).expect("解析日志失败");
        println!("推送tag失败: {}", msg);
    }
}

// 创建tag
fn create_git_tag(version: &str, path: &str) {
    let commit = get_git_commit_latest(path);
    let output = Command::new("git")
        .arg("tag")
        .arg(format!("V{}", version))
        .arg(commit)
        .current_dir(path)
        .output()
        .expect("创建tag失败");
    if output.status.success() {
        println!("{}  V{}", path, version);
    } else {
        let msg = String::from_utf8(output.stdout).expect("解析日志失败");
        println!("创建tag失败!, {}", msg);
    }
}

// 创建版本
fn version_add_one(vtype: i32, version: &str) -> String {
    let version = version.to_string();
    let version = version.split(".");
    let version = version.collect::<Vec<&str>>();
    let mut version = version
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let mut version = version
        .iter_mut()
        .map(|x| x.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    if vtype == 1 {
        version[0] += 1;
        version[1] = 0;
        version[2] = 0;
    } else if vtype == 2 {
        version[1] += 1;
        version[2] = 0;
    } else if vtype == 3 {
        version[2] += 1;
    }
    let version = version
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let version = version.join(".");
    version
}

// 获取tag对应的commit
fn get_git_tagcommit_latest(tag: &str, path: &str) -> String {
    let output = Command::new("git")
        .arg("rev-list")
        .arg("-n")
        .arg("1")
        .arg(tag)
        .current_dir(path)
        .output()
        .expect("获取tag对应的commit失败");
    let commit = String::from_utf8(output.stdout).expect("解析日志失败");
    if commit == "" {
        return "".to_string();
    }
    commit.trim().to_string()
}

// 获取最新tag
fn get_git_tag_latest(path: &str) -> String {
    let output = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--abbrev=0")
        .arg("master")
        .current_dir(path)
        .output()
        .expect("执行 git describe 失败");
    if output.status.success() {
        let tag = String::from_utf8(output.stdout).expect("解析日志失败");
        tag.trim().to_string()
    } else {
        String::from("V0.0.0")
    }
}

// 获取最新commit
fn get_git_commit_latest(path: &str) -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("master")
        .current_dir(path)
        .output()
        .expect("执行 git rev-parse 失败");
    if output.status.success() {
        let commit = String::from_utf8(output.stdout).expect("解析日志失败");
        commit.trim().to_string()
    } else {
        String::from("0000000")
    }
}

// 检查是否是git仓库
fn is_git_repository(path: &str) -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(path)
        .output()
        .expect("执行 git rev-parse 失败");
    if output.status.success() {
        let is_git = String::from_utf8(output.stdout).expect("解析日志失败");
        is_git.trim().to_string()
    } else {
        String::from("false")
    }
}
