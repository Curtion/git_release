use crate::{json, model};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use microkv::MicroKV;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use std::{env, fs, thread};

// 开始部署任务
pub fn deploy_job(deploy_list: Vec<json::Deploy>) {
    let config = parse_user_toml();
    if let Err(error) = config {
        println!("{error}");
        return;
    }
    let config = config.unwrap();
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
            let job_name = Path::new(&item.path).file_name().unwrap().to_str().unwrap();
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

// 解析配置文件
fn parse_user_toml() -> Result<json::Config, Box<dyn Error>> {
    let exe_path = env::current_exe().expect("获取当前路径失败");
    let exe_path = exe_path.to_str().unwrap();
    let exe_dir = Path::new(exe_path).parent().unwrap();
    let toml_str = fs::read_to_string(exe_dir.join("user.toml"));
    let toml_str = match toml_str {
        Ok(toml_str) => toml_str,
        Err(_) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "程序运行目录未找到配置文件user.toml",
            )))
        }
    };
    let config: json::Config = toml::from_str(&toml_str).unwrap();
    return Ok(config);
}

// 初始持久化db
fn init_user_db() -> MicroKV {
    let unsafe_pwd: String = String::from("jidian@iot");
    let mut db = MicroKV::open("user_db").unwrap().with_pwd_clear(unsafe_pwd);
    db = db.set_auto_commit(true);
    return db;
}

// 查询华为云任务列表
#[tokio::main]
async fn get_huawei_jobs(
    db: &MicroKV,
    config: &json::Config,
) -> Result<model::ProjectList, Box<dyn std::error::Error>> {
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
        .json::<model::ProjectList>()
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
async fn huawei_login(
    db: &MicroKV,
    config: &json::Config,
) -> Result<(), Box<dyn std::error::Error>> {
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
) -> Result<model::JobDetail, Box<dyn std::error::Error>> {
    let json = model::BuildJob {
        job_id: jobid.to_string(),
        scm: model::Scm {
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
        let resjson = res.json::<model::JobDetail>().await?;
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
) -> Result<model::JobResult, Box<dyn std::error::Error>> {
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
        Ok(res.json::<model::JobResult>().await?)
    } else {
        Err(res.text().await?.into())
    }
}

fn is_success(job_result: &model::JobResult) -> bool {
    job_result
        .build_steps
        .iter()
        .all(|item| item.status == "success")
}

fn is_error(job_result: &model::JobResult) -> bool {
    job_result
        .build_steps
        .iter()
        .any(|item| item.status == "error")
}
