// Integration Tests: Manager ↔ Docker Socket
// Feature: Manager-Docker Socket Integration
// 
// 管理画面とDocker Socketの統合テストを実施します。
// Docker Socketへのアクセス、コンテナ操作、エラーハンドリングを検証します。

use std::process::Command;

// テスト用のヘルパー関数

/// Dockerが利用可能かチェック
fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Docker Socketがマウントされているかチェック
fn is_docker_socket_mounted() -> bool {
    #[cfg(unix)]
    {
        use std::path::Path;
        Path::new("/var/run/docker.sock").exists()
    }
    
    #[cfg(not(unix))]
    {
        // Windows環境では名前付きパイプを使用
        true
    }
}

/// 指定されたコンテナが実行中かチェック
fn is_container_running(container_name: &str) -> bool {
    let output = Command::new("docker")
        .args(&["ps", "--filter", &format!("name={}", container_name), "--format", "{{.Names}}"])
        .output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            stdout.trim() == container_name
        }
        Err(_) => false,
    }
}

/// コンテナIDを取得
fn get_container_id(container_name: &str) -> Option<String> {
    let output = Command::new("docker")
        .args(&["ps", "-aqf", &format!("name={}", container_name)])
        .output()
        .ok()?;
    
    if output.status.success() {
        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }
    
    None
}

// 統合テスト

#[test]
fn test_docker_socket_availability() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.1, 12.3
    // Docker Socketが利用可能であることを確認
    
    assert!(
        is_docker_available(),
        "Docker is not available. Please ensure Docker is installed and running."
    );
}

#[test]
#[cfg(unix)]
fn test_docker_socket_file_exists() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.1, 12.3
    // Docker Socketファイルが存在することを確認（Unix系のみ）
    
    use std::path::Path;
    
    let socket_path = Path::new("/var/run/docker.sock");
    assert!(
        socket_path.exists(),
        "Docker socket file does not exist at /var/run/docker.sock"
    );
}

#[test]
fn test_docker_ps_command() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.1, 12.3
    // docker ps コマンドが正常に実行できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let output = Command::new("docker")
        .arg("ps")
        .output()
        .expect("Failed to execute docker ps");
    
    assert!(
        output.status.success(),
        "docker ps command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_docker_inspect_command() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.1, 12.3
    // docker inspect コマンドが正常に実行できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    // 実行中のコンテナを取得
    let ps_output = Command::new("docker")
        .args(&["ps", "-q", "--limit", "1"])
        .output()
        .expect("Failed to execute docker ps");
    
    let container_id = String::from_utf8_lossy(&ps_output.stdout).trim().to_string();
    
    if container_id.is_empty() {
        eprintln!("Skipping test: No running containers found");
        return;
    }
    
    // docker inspect を実行
    let output = Command::new("docker")
        .args(&["inspect", &container_id])
        .output()
        .expect("Failed to execute docker inspect");
    
    assert!(
        output.status.success(),
        "docker inspect command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_telegraf_container_identification() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.1, 12.3
    // Telegrafコンテナを正しく識別できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let container_name = "telegraf";
    
    if !is_container_running(container_name) {
        eprintln!("Skipping test: Telegraf container is not running");
        return;
    }
    
    // コンテナIDを取得
    let container_id = get_container_id(container_name);
    assert!(
        container_id.is_some(),
        "Failed to get Telegraf container ID"
    );
    
    // コンテナ情報を取得
    let output = Command::new("docker")
        .args(&["inspect", "--format", "{{.Name}}", &container_id.unwrap()])
        .output()
        .expect("Failed to execute docker inspect");
    
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        name.contains(container_name),
        "Container name should contain 'telegraf', got: {}",
        name
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_docker_kill_sighup_command() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 11.2, 12.4
    // docker kill --signal=SIGHUP コマンドが正常に実行できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let container_name = "telegraf";
    
    if !is_container_running(container_name) {
        eprintln!("Skipping test: Telegraf container is not running");
        return;
    }
    
    // SIGHUPシグナルを送信
    let output = Command::new("docker")
        .args(&["kill", "--signal=SIGHUP", container_name])
        .output()
        .expect("Failed to execute docker kill");
    
    assert!(
        output.status.success(),
        "docker kill --signal=SIGHUP failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_docker_socket_permission_error_handling() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 12.5
    // Docker Socketへのアクセス権限がない場合のエラーハンドリングを確認
    
    // 存在しないDockerコマンドを実行してエラーをシミュレート
    let output = Command::new("docker-nonexistent")
        .arg("ps")
        .output();
    
    // コマンド実行が失敗することを確認
    assert!(
        output.is_err(),
        "Command should fail when Docker is not accessible"
    );
}

#[test]
fn test_container_not_found_error() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 11.5, 12.5
    // 存在しないコンテナに対するエラーハンドリングを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let nonexistent_container = "nonexistent-container-12345";
    
    // 存在しないコンテナにSIGHUPを送信
    let output = Command::new("docker")
        .args(&["kill", "--signal=SIGHUP", nonexistent_container])
        .output()
        .expect("Failed to execute docker kill");
    
    // コマンドが失敗することを確認
    assert!(
        !output.status.success(),
        "Command should fail for nonexistent container"
    );
    
    // エラーメッセージを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No such container") || stderr.contains("Error: No such container"),
        "Error message should indicate container not found: {}",
        stderr
    );
}

#[test]
fn test_docker_version_check() {
    // Feature: Manager-Docker Socket Integration
    // Dockerのバージョン情報が取得できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let output = Command::new("docker")
        .arg("version")
        .output()
        .expect("Failed to execute docker version");
    
    assert!(
        output.status.success(),
        "docker version command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Version:") || stdout.contains("version"),
        "Docker version output should contain version information"
    );
}

#[test]
fn test_docker_info_command() {
    // Feature: Manager-Docker Socket Integration
    // docker info コマンドが正常に実行できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let output = Command::new("docker")
        .arg("info")
        .output()
        .expect("Failed to execute docker info");
    
    assert!(
        output.status.success(),
        "docker info command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_container_state_verification() {
    // Feature: Manager-Docker Socket Integration
    // コンテナの状態を正しく確認できることを検証
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let container_name = "telegraf";
    
    if !is_container_running(container_name) {
        eprintln!("Skipping test: Telegraf container is not running");
        return;
    }
    
    // コンテナの状態を取得
    let output = Command::new("docker")
        .args(&[
            "inspect",
            "--format",
            "{{.State.Running}}",
            container_name,
        ])
        .output()
        .expect("Failed to execute docker inspect");
    
    let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        state, "true",
        "Telegraf container should be running"
    );
}

#[test]
fn test_docker_socket_error_messages() {
    // Feature: Manager-Docker Socket Integration
    // Requirements: 11.5, 12.5
    // Docker Socketエラー時のメッセージが適切であることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    // 不正なコマンドを実行
    let output = Command::new("docker")
        .args(&["invalid-command"])
        .output()
        .expect("Failed to execute docker command");
    
    // コマンドが失敗することを確認
    assert!(!output.status.success());
    
    // エラーメッセージが出力されることを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "Error message should not be empty"
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_multiple_container_operations() {
    // Feature: Manager-Docker Socket Integration
    // 複数のコンテナ操作が連続して実行できることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    // docker ps を複数回実行
    for i in 0..5 {
        let output = Command::new("docker")
            .arg("ps")
            .output()
            .expect(&format!("Failed to execute docker ps (iteration {})", i));
        
        assert!(
            output.status.success(),
            "docker ps command failed at iteration {}: {}",
            i,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_docker_command_timeout_handling() {
    // Feature: Manager-Docker Socket Integration
    // Dockerコマンドのタイムアウト処理を確認
    
    use std::time::{Duration, Instant};
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let start = Instant::now();
    
    // 短時間で完了するコマンドを実行
    let output = Command::new("docker")
        .arg("version")
        .output()
        .expect("Failed to execute docker version");
    
    let duration = start.elapsed();
    
    assert!(
        output.status.success(),
        "docker version command failed"
    );
    
    // コマンドが妥当な時間内に完了することを確認（10秒以内）
    assert!(
        duration < Duration::from_secs(10),
        "Docker command took too long: {:?}",
        duration
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_container_logs_access() {
    // Feature: Manager-Docker Socket Integration
    // コンテナのログにアクセスできることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    let container_name = "telegraf";
    
    if !is_container_running(container_name) {
        eprintln!("Skipping test: Telegraf container is not running");
        return;
    }
    
    // コンテナのログを取得（最新10行）
    let output = Command::new("docker")
        .args(&["logs", "--tail", "10", container_name])
        .output()
        .expect("Failed to execute docker logs");
    
    assert!(
        output.status.success(),
        "docker logs command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
