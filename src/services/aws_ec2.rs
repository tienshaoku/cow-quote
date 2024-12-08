pub fn is_running_in_aws_ec2() -> bool {
    let output = std::process::Command::new("curl")
        .arg("--connect-timeout")
        .arg("2") // 2 second timeout
        .arg("http://169.254.169.254/latest/meta-data/instance-id")
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
