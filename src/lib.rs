use std::process::Command;
use std::process::Output;

pub fn git_cmd_out<'a>(working_dir: String, args: &[&str]) -> Output {
    return Command::new("git")
        .args(&["-C", &working_dir])
        .args(args)
        .output()
        .expect("Failed to execute git command");
}

pub fn tags_from_url(url: &str) -> Result<Vec<String>, &'static str> {
    let mut vec = Vec::new();
    let output = Command::new("git")
        .args(&["ls-remote", "--refs", "--tags"])
        .arg(url)
        .output()
        .expect("Failed to execute git ls-remote");
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).unwrap();
        for s in tmp.lines() {
            let mut split = s.splitn(3, "/");
            split.next();
            split.next();
            let split_result = split.next();
            if split_result.is_some() {
                vec.push(String::from(split_result.unwrap()));
            }
        }
        return Ok(vec);
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn top_level() -> Result<String, &'static str> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to execute git rev-parse");
    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)
            .unwrap()
            .trim_end()
            .to_string());
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn config_set(
    working_dir: &str,
    file: &str,
    key: &str,
    value: &str,
) -> Result<bool, &'static str> {
    let output = Command::new("git")
        .args(&["-C", working_dir])
        .args(&["config", "--file", file, key, value])
        .output()
        .expect("Failed to execute git config");
    if output.status.success() {
        return Ok(true);
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn sparse_checkout_add(working_dir: &str, target: &str) -> Result<bool, &'static str> {
    let output = Command::new("git")
        .args(&["-C", working_dir])
        .args(&["sparse-checkout", "add", target])
        .output()
        .expect("Failed to execute git sparse-checkout");
    if output.status.success() {
        return Ok(true);
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn is_sparse(working_dir: &str) -> bool {
    let output = Command::new("git")
        .args(&["-C", working_dir])
        .args(&["config", "core.sparseCheckout"])
        .output()
        .expect("Failed to execute git config");

    return String::from_utf8(output.stdout).unwrap() == "true";
}

pub fn subtree_add(
    working_dir: &str,
    target: &str,
    url: &str,
    git_ref: &str,
    msg: &str,
) -> Result<bool, &'static str> {
    let output = Command::new("git")
        .args(&["-C", working_dir])
        .args(&["subtree", "add", "-P", target, url, git_ref, "-m", msg])
        .output()
        .expect("Failed to execute git subtree");
    if output.status.success() {
        return Ok(true);
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn subtree_files(working_dir: &str) -> Result<Vec<String>, &'static str> {
    let output = git_cmd_out(
        working_dir.to_string(),
        &["ls-files", "--", "*.gitsubtrees"],
    );
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).unwrap();
        return Ok(tmp.lines().map(str::to_string).collect());
    } else {
        let tmp = String::from_utf8(output.stderr).unwrap();
        return Err(Box::leak(tmp.into_boxed_str()));
    }
}

pub fn is_working_dir_clean(working_dir: &str) -> bool {
    let output = git_cmd_out(working_dir.to_string(), &["diff", "--quiet"]);
    return output.status.success();
}

pub fn resolve_head(url: &str) -> Result<String, String> {
    let proc = Command::new("git")
        .args(&["ls-remote", "--symref", url, "HEAD"])
        .output()
        .expect("Failed to execute git command");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).unwrap();
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse HEAD from remote");
        let mut split = first_line
            .split("\t")
            .next()
            .expect("Failed to parse HEAD from remote")
            .splitn(3, "/");
        split.next();
        split.next();
        return Ok(split.next().unwrap().to_string());
    }

    let tmp = String::from_utf8(proc.stderr).unwrap();
    return Err(tmp);
}

pub fn remote_ref_to_id(url: &str, name: &str) -> Result<String, String> {
    let proc = Command::new("git")
        .args(&["ls-remote", url, name])
        .output()
        .expect("Failed to execute git ls-remote");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).unwrap();
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse id from remote");
        return Ok(first_line.split("\t").next().unwrap().to_string());
    }
    let tmp = String::from_utf8(proc.stderr).unwrap();
    return Err(tmp);
}

pub fn short_ref(working_dir: &str, long_ref: &str) -> Result<String, String> {
    let proc = git_cmd_out(working_dir.to_string(), &["rev-parse", "--short", long_ref]);
    if proc.status.success() {
        return Ok(String::from_utf8(proc.stdout)
            .unwrap()
            .trim_end()
            .to_string());
    }
    let tmp = String::from_utf8(proc.stderr).unwrap();
    return Err(tmp);
}
