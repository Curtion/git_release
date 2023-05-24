mod git;
mod huawei;
mod json;
mod model;

use clap::Parser;
use std::{cmp::Ordering, env, fs, io};

fn main() {
    let args = json::Args::parse();
    let path = if args.path.is_empty() {
        env::current_dir()
            .unwrap()
            .to_str()
            .unwrap_or_default()
            .to_string()
    } else {
        args.path
    };
    let dirs = fs::read_dir(path).unwrap();
    let mut latest = json::Repository {
        paths: Vec::new(),
        tags: Vec::new(),
    };
    let mut old = json::Repository {
        paths: Vec::new(),
        tags: Vec::new(),
    };
    dirs.filter_map(Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && git::is_git_repository(e.path().to_str().unwrap_or_default()) == "true"
        })
        .fold((&mut latest, &mut old), |acc, path| {
            let path = path.path();
            let path = path.to_str().unwrap_or_default();
            let tag = git::get_git_tag_latest(path);
            let commit: String = git::get_git_commit_latest(path);
            let tagcommit = git::get_git_tagcommit_latest(&tag, path);
            if !tagcommit.starts_with(&commit) {
                println!("路径: {}", path);
                println!("最新tag: {}", tag);
                println!("最新commit: {}", commit);
                println!("最新tag对应的commit: {}", tagcommit);
                println!("---------------------------------------------");
                acc.0.paths.push(path.to_string());
                acc.0.tags.push(tag.to_string());
            } else {
                acc.1.paths.push(path.to_string());
                acc.1.tags.push(tag.to_string());
            }
            acc
        });
    if latest.paths.len() == 0 {
        println!("没有需要更新的项目!以下为当前最新tag:");
        println!("---------------------------------------------");
        for i in 0..old.paths.len() {
            println!("{} {}", old.paths[i], old.tags[i]);
        }
        return;
    }
    let mut input_type: i32;
    println!("请输入要更新的版本类型!");
    println!("1. 大版本");
    println!("2. 小版本");
    println!("3. 修复版本");
    println!("4. 退出");
    loop {
        let mut input: String = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input_type = input.trim().parse::<i32>().unwrap_or_default();
        if input_type <= 0 || input_type > 4 {
            println!("输入错误!, 请重新输入!");
        } else {
            break;
        }
    }
    let mut deploy_list: Vec<json::Deploy> = Vec::new();
    for (index, path) in latest.paths.iter().enumerate() {
        let last_tag = latest
            .tags
            .get(index)
            .unwrap()
            .split("V")
            .collect::<Vec<&str>>();
        let last_tag = last_tag.get(1).unwrap_or(&"0.0.0");
        let version = version_add_one(input_type, last_tag);
        git::create_git_tag(&version, path);
        git::git_push_tag(path);
        deploy_list.push(json::Deploy {
            path: path.to_string(),
            tag: version.to_string(),
        });
    }
    println!("是否启动华为云部署? y/N");
    loop {
        let mut input_type = String::new();
        io::stdin()
            .read_line(&mut input_type)
            .expect("输入解析错误!");
        let input_type = input_type.trim();
        match input_type {
            "y" | "Y" => {
                huawei::deploy_job(deploy_list.clone());
                return;
            },
            "n" | "N" => {
                println!("已取消华为云部署!");
                return;
            },
            _ => println!("输入错误, 请重新输入!"),
        }
    }
}

// 获取下一个版本号
fn version_add_one(vtype: i32, version: &str) -> String {
    version
        .split(".")
        .map(|e| e.parse::<i32>().unwrap_or_default())
        .enumerate()
        .map(|(i, x)| match (i + 1).cmp(&(vtype as usize)) {
            Ordering::Equal => x + 1,
            Ordering::Less => x,
            Ordering::Greater => 0,
        })
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(".")
}
