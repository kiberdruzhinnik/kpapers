use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::{Command, Stdio},
};

#[test]
fn initialization_failure_exits_nonzero_with_only_a_structured_stdout_event() {
    let database =
        std::env::temp_dir().join(format!("kpapers-cli-log-{}.sqlite", uuid::Uuid::new_v4()));
    let output = Command::new(env!("CARGO_BIN_EXE_kpapers-api"))
        .env(
            "DATABASE_URL",
            format!("sqlite://{}?mode=rwc", database.display()),
        )
        .env("OPERATOR_PASSWORD_HASH", "unused-in-startup-test")
        .env_remove("SESSION_SIGNING_KEY")
        .output()
        .expect("run the API binary");

    assert!(
        !output.status.success(),
        "startup failure must exit nonzero"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let events: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("stdout log line is JSON"))
        .collect();
    assert_eq!(events.len(), 1, "stdout: {stdout}");
    assert_eq!(events[0]["fields"]["event"], "api_initialization_failed");
    assert_eq!(
        events[0]["fields"]["error"],
        "SESSION_SIGNING_KEY must be configured"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        !stderr.contains("api_initialization_failed"),
        "stderr duplicated the structured event: {stderr}"
    );
    assert!(
        !stderr.contains("SESSION_SIGNING_KEY must be configured"),
        "stderr duplicated the startup error: {stderr}"
    );

    for suffix in ["", "-wal", "-shm"] {
        let mut file = database.as_os_str().to_os_string();
        file.push(suffix);
        let _ = fs::remove_file(file);
    }
}

#[test]
fn failing_healthcheck_exits_nonzero_with_a_diagnostic() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test healthcheck server");
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0u8; 1024];
                    let _ = stream.read(&mut request);
                    let _ = stream.write_all(
                        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "healthcheck did not connect"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => panic!("healthcheck server failed: {error}"),
            }
        }
    });

    let output = Command::new(env!("CARGO_BIN_EXE_kpapers-api"))
        .arg("--healthcheck")
        .env(
            "HEALTHCHECK_URL",
            format!("http://127.0.0.1:{port}/healthz"),
        )
        .output()
        .expect("run the API binary");
    server.join().expect("healthcheck server");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.contains("API health check failed with status 503"),
        "stderr: {stderr}"
    );
}

#[test]
fn failing_hash_password_exits_nonzero_with_a_diagnostic() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kpapers-api"))
        .arg("--hash-password")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the API binary");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"\n")
        .expect("write empty password");
    let output = child.wait_with_output().expect("wait for the API binary");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.contains("password must not be empty"),
        "stderr: {stderr}"
    );
}
