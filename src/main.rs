use clap::Parser;
use std::process::Command;
use std::{env, fs, io};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "")]
    path: String,
}

fn main() {
    let args = Args::parse();
    let mut path = args.path;
    if path == "" {
        let dir = env::current_dir().expect("获取当前目录失败");
        path = dir.to_str().unwrap().to_string();
    }
    let dirs = fs::read_dir(path).expect("读取目录失败");
    let mut paths: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    for entry in dirs {
        if let Ok(entry) = entry {
            let path = entry.path();
            let is_dir = path.is_dir();
            let path = path.to_str().expect("解析目录失败");
            if is_dir {
                let is_git = is_git_repository(path);
                if is_git == "true" {
                    let tag = get_git_tag_latest(path);
                    let commit = get_git_commit_latest(path);
                    let tagcommit = get_git_tagcommit_latest(&tag, path);
                    if tagcommit != commit {
                        println!("路径: {}", path);
                        println!("最新tag: {}", tag);
                        println!("最新commit: {}", commit);
                        println!("最新tag对应的commit: {}", tagcommit);
                        println!("---------------------------------------------");
                        paths.push(path.to_string());
                        tags.push(tag.to_string());
                    }
                }
            }
        }
    }
    if paths.len() == 0 {
        println!("没有需要更新的项目!");
        return;
    }
    let mut input_type = String::new();
    println!("请输入要更新的版本类型!");
    println!("1. 大版本");
    println!("2. 小版本");
    println!("3. 修复版本");
    io::stdin()
        .read_line(&mut input_type)
        .expect("输入解析错误!");
    let input_type = input_type.trim().parse::<i32>().expect("输入格式错误!");
    for (index, path) in paths.iter().enumerate() {
        let last_tag = tags
            .get(index)
            .expect("未知错误")
            .split("V")
            .collect::<Vec<&str>>()[1];
        let version = version_add_one(input_type, last_tag);
        create_git_tag(&version, path);
        git_push_tag(path);
    }
}

fn git_push_tag(path: &str) {
    let output = Command::new("git")
        .arg("push")
        .arg("origin")
        .arg("--tags")
        .current_dir(path)
        .output()
        .expect("执行git push tag失败");
    if output.status.success() {
        println!("推送tag成功!");
    } else {
        let msg = String::from_utf8(output.stdout).expect("解析日志失败");
        println!("推送tag失败!, {}", msg);
    }
}

// 创建tag
fn create_git_tag(version: &str, path: &str) {
    let output = Command::new("git")
        .arg("tag")
        .arg(format!("V{}", version))
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
    commit.trim()[..7].to_string()
}

// 获取最新tag
fn get_git_tag_latest(path: &str) -> String {
    let output = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--abbrev=0")
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
        .arg("HEAD")
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
