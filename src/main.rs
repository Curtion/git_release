mod git;
mod huawei;
mod json;
mod model;

use clap::Parser;
use std::{env, fs, io};

fn main() {
    version_add_one(3, "1.2.3");
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
                let is_git = git::is_git_repository(path);
                if is_git == "true" {
                    let tag = git::get_git_tag_latest(path);
                    let commit: String = git::get_git_commit_latest(path);
                    let tagcommit = git::get_git_tagcommit_latest(&tag, path);
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
        git::create_git_tag(&version, path);
        git::git_push_tag(path);
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
        huawei::deploy_job(deploy_list);
    } else {
        println!("已取消华为云部署!");
    }
}

// 获取下一个版本号
fn version_add_one(vtype: i32, version: &str) -> String {
    version
        .split(".")
        .map(|e| e.parse::<i32>().unwrap_or_default())
        .enumerate()
        .map(|(i, x)| {
            if (i + 1) == vtype as usize {
                x + 1
            } else {
                if (i + 1) > vtype as usize {
                    0
                } else {
                    x
                }
            }
        })
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(".")
}
