//! End-to-end integration tests for the pixelensd daemon.
//!
//! These spawn the actual `pixelensd` binary against a controlled
//! environment (isolated XDG_RUNTIME_DIR, stub slurp/grim in a
//! private $PATH) and exercise the real Unix-socket IPC. They are
//! the closest we can get to a real Wayland session from CI.

#[cfg(unix)]
mod unix_integration {
    use std::env;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;

    use pixelens_ipc::{
        read_response, write_frame, Command as IpcCommand, GrabResponsePayload, IpcRequest,
        IpcResponse, ResponseStatus,
    };
    use tokio::net::UnixStream;
    use uuid::Uuid;

    /// Isolated test environment: a temp runtime dir plus a stub $PATH
    /// with fake slurp / grim so the pipeline can be driven from CI.
    struct IsolatedEnv {
        base: PathBuf,
        runtime_dir: PathBuf,
        bin_dir: PathBuf,
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl IsolatedEnv {
        fn new() -> Self {
            let base = std::env::temp_dir().join(format!("pixelens-it-{}", Uuid::new_v4()));
            let runtime_dir = base.join("run");
            let bin_dir = base.join("bin");
            std::fs::create_dir_all(&runtime_dir).unwrap();
            std::fs::create_dir_all(&bin_dir).unwrap();

            let saved = vec![
                ("XDG_RUNTIME_DIR", env::var("XDG_RUNTIME_DIR").ok()),
                ("PATH", env::var("PATH").ok()),
            ];
            env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
            env::set_var("PATH", &bin_dir);

            Self {
                base,
                runtime_dir,
                bin_dir,
                saved,
            }
        }

        fn install_stub(&self, name: &str, body: &str) {
            let p = self.bin_dir.join(name);
            std::fs::write(&p, body).unwrap();
            std::fs::set_permissions(&p, PermissionsExt::from_mode(0o755)).unwrap();
        }

        fn socket_path(&self) -> PathBuf {
            self.runtime_dir.join("pixelens.sock")
        }
    }

    impl Drop for IsolatedEnv {
        fn drop(&mut self) {
            for (k, v) in self.saved.iter() {
                match v {
                    Some(val) => env::set_var(k, val),
                    None => env::remove_var(k),
                }
            }
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    fn spawn_daemon(env: &IsolatedEnv) -> std::process::Child {
        let bin = env!("CARGO_BIN_EXE_pixelensd");
        let child = Command::new(bin)
            .env("XDG_RUNTIME_DIR", &env.runtime_dir)
            .env("PATH", &env.bin_dir)
            .env("WAYLAND_DISPLAY", "wayland-99")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("failed to spawn pixelensd");
        wait_for_socket(&env.socket_path(), Duration::from_secs(5));
        child
    }

    fn wait_for_socket(path: &std::path::Path, timeout: Duration) {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if path.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!(
            "daemon socket did not appear at {} within {:?}",
            path.display(),
            timeout
        );
    }

    async fn send_command(socket: &std::path::Path, command: IpcCommand) -> IpcResponse {
        let mut stream = UnixStream::connect(socket).await.expect("connect");
        let req = IpcRequest {
            request_id: Uuid::new_v4().to_string(),
            command,
            payload: serde_json::json!({}),
        };
        write_frame(&req, &mut stream).await.expect("write");
        read_response(&mut stream).await.expect("read")
    }

    #[tokio::test]
    async fn status_round_trip() {
        let env = IsolatedEnv::new();
        env.install_stub("slurp", "#!/bin/sh\nexit 1\n");
        env.install_stub("grim", "#!/bin/sh\nexit 1\n");

        let mut child = spawn_daemon(&env);
        let resp = send_command(&env.socket_path(), IpcCommand::Status).await;
        assert_eq!(resp.status, ResponseStatus::Ok);
        assert_eq!(resp.payload["display"], "wayland");
        assert_eq!(resp.payload["pipeline_ready"], true);

        let _ = child.kill();
        let _ = child.wait();
    }

    #[tokio::test]
    async fn grab_cancelled_when_slurp_exits_empty() {
        let env = IsolatedEnv::new();
        env.install_stub("slurp", "#!/bin/sh\nexit 1\n");
        env.install_stub("grim", "#!/bin/sh\nexit 1\n");

        let mut child = spawn_daemon(&env);
        let resp = send_command(&env.socket_path(), IpcCommand::Grab).await;
        assert_eq!(resp.status, ResponseStatus::Cancelled);

        let _ = child.kill();
        let _ = child.wait();
    }

    #[tokio::test]
    async fn grab_captured_end_to_end() {
        let env = IsolatedEnv::new();
        env.install_stub("slurp", "#!/bin/sh\nprintf '320x180+10+20'\n");
        env.install_stub(
            "grim",
            // Write a 1024-byte capture file using only POSIX shell
            // builtins. The isolated $PATH in this test contains only the
            // stubs, so external tools like `head`/`dd` are NOT available
            // and must not be used here.
            "#!/bin/sh\nout=\"$3\"\ni=0\nwhile [ $i -lt 1024 ]; do printf '\\0'; i=$((i+1)); done > \"$out\"\n",
        );

        let mut child = spawn_daemon(&env);
        let resp = send_command(&env.socket_path(), IpcCommand::Grab).await;
        assert_eq!(resp.status, ResponseStatus::Ok);

        let payload: GrabResponsePayload = serde_json::from_value(resp.payload).unwrap();
        assert_eq!(payload.region.width, 320);
        assert_eq!(payload.region.height, 180);
        assert_eq!(payload.region.x, 10);
        assert_eq!(payload.region.y, 20);
        assert!(payload.bytes >= 1024);
        let p = PathBuf::from(&payload.path);
        assert!(p.exists(), "capture file should exist: {}", p.display());

        let _ = child.kill();
        let _ = child.wait();
    }

    #[tokio::test]
    async fn grab_reports_missing_tool_when_slurp_not_installed() {
        let env = IsolatedEnv::new();
        // Only grim is present; the upfront which() check for slurp fails.
        env.install_stub("grim", "#!/bin/sh\nexit 1\n");

        let mut child = spawn_daemon(&env);
        let resp = send_command(&env.socket_path(), IpcCommand::Grab).await;

        assert_eq!(resp.status, ResponseStatus::Error);
        let msg = resp
            .payload
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("slurp") || msg.contains("capture pipeline"),
            "unexpected error message: {msg}"
        );

        let _ = child.kill();
        let _ = child.wait();
    }
}
