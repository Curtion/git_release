// 推送tag
use std::process::Command;

pub fn git_push_tag(path: &str) {
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
pub fn create_git_tag(version: &str, path: &str) {
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

// 获取最新commit
pub fn get_git_commit_latest(path: &str) -> String {
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

// 获取tag对应的commit
pub fn get_git_tagcommit_latest(tag: &str, path: &str) -> String {
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
pub fn get_git_tag_latest(path: &str) -> String {
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

// 检查是否是git仓库
pub fn is_git_repository(path: &str) -> String {
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