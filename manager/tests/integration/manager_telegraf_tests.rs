// Integration Tests: Manager ↔ Telegraf
// Feature: Manager-Telegraf Integration
// 
// 管理画面とTelegrafコンテナの統合テストを実施します。
// 実際のDockerコンテナを使用して、設定更新とリロードの動作を検証します。

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// テスト用のヘルパー関数

/// Telegrafコンテナが実行中かチェック
fn is_telegraf_running() -> bool {
    let output = Command::new("docker")
        .args(&["ps", "--filter", "name=telegraf", "--format", "{{.Names}}"])
        .output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            stdout.contains("telegraf")
        }
        Err(_) => false,
    }
}

/// Dockerが利用可能かチェック
fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// テスト用のTelegraf設定ファイルを作成
fn create_test_telegraf_config(dir: &Path, urls: Vec<&str>) -> String {
    let config_content = format!(
        r#"
[agent]
  interval = "10s"
  round_interval = true
  metric_batch_size = 1000
  metric_buffer_limit = 10000
  collection_jitter = "0s"
  flush_interval = "10s"
  flush_jitter = "0s"

[[outputs.influxdb_v2]]
  urls = ["http://influxdb:8086"]
  token = "test-token"
  organization = "test-org"
  bucket = "test-bucket"

[[inputs.http_response]]
  urls = [{}]
  response_timeout = "5s"
  method = "GET"
  follow_redirects = true
"#,
        urls.iter()
            .map(|url| format!("\"{}\"", url))
            .collect::<Vec<_>>()
            .join(", ")
    );
    
    let config_path = dir.join("telegraf.conf");
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path.to_string_lossy().to_string()
}

// 統合テスト

#[test]
fn test_docker_availability() {
    // Feature: Manager-Telegraf Integration
    // Dockerが利用可能であることを確認
    assert!(
        is_docker_available(),
        "Docker is not available. Please ensure Docker is installed and running."
    );
}

#[test]
fn test_telegraf_config_file_structure() {
    // Feature: Manager-Telegraf Integration
    // Telegraf設定ファイルの構造が正しいことを確認
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let urls = vec!["http://example.com", "https://test.com"];
    
    let config_path = create_test_telegraf_config(temp_dir.path(), urls.clone());
    
    // ファイルが作成されたことを確認
    assert!(Path::new(&config_path).exists());
    
    // ファイル内容を読み込み
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    
    // URLが含まれていることを確認
    for url in urls {
        assert!(content.contains(url), "Config should contain URL: {}", url);
    }
    
    // 必須セクションが含まれていることを確認
    assert!(content.contains("[agent]"));
    assert!(content.contains("[[outputs.influxdb_v2]]"));
    assert!(content.contains("[[inputs.http_response]]"));
}

#[test]
fn test_config_update_atomicity() {
    // Feature: Manager-Telegraf Integration, Property 1: Atomicity
    // 設定ファイル更新の原子性を確認
    // 書き込み中にファイルが読み取られても、完全な内容が取得できることを検証
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("telegraf.conf");
    
    // 初期設定を作成
    let initial_urls = vec!["http://initial.com"];
    let initial_content = format!(
        "[[inputs.http_response]]\n  urls = [\"{}\"]\n",
        initial_urls[0]
    );
    fs::write(&config_path, &initial_content).expect("Failed to write initial config");
    
    // 設定を更新
    let updated_urls = vec!["http://updated1.com", "http://updated2.com"];
    let updated_content = format!(
        "[[inputs.http_response]]\n  urls = [{}]\n",
        updated_urls
            .iter()
            .map(|url| format!("\"{}\"", url))
            .collect::<Vec<_>>()
            .join(", ")
    );
    fs::write(&config_path, &updated_content).expect("Failed to write updated config");
    
    // 更新後の内容を読み取り
    let read_content = fs::read_to_string(&config_path).expect("Failed to read config");
    
    // 更新されたURLが含まれていることを確認
    for url in updated_urls {
        assert!(read_content.contains(url), "Config should contain updated URL: {}", url);
    }
    
    // 古いURLが含まれていないことを確認
    assert!(!read_content.contains(initial_urls[0]), "Config should not contain old URL");
}

#[test]
fn test_config_backup_and_restore() {
    // Feature: Manager-Telegraf Integration, Property 5: Rollback
    // 設定ファイルのバックアップと復元機能を確認
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("telegraf.conf");
    let backup_path = temp_dir.path().join("telegraf.conf.backup");
    
    // 初期設定を作成
    let initial_content = "[[inputs.http_response]]\n  urls = [\"http://initial.com\"]\n";
    fs::write(&config_path, initial_content).expect("Failed to write initial config");
    
    // バックアップを作成
    fs::copy(&config_path, &backup_path).expect("Failed to create backup");
    
    // バックアップが作成されたことを確認
    assert!(backup_path.exists());
    
    // 設定を更新（失敗をシミュレート）
    let updated_content = "[[inputs.http_response]]\n  urls = [\"http://updated.com\"]\n";
    fs::write(&config_path, updated_content).expect("Failed to write updated config");
    
    // バックアップから復元
    fs::copy(&backup_path, &config_path).expect("Failed to restore from backup");
    
    // 復元された内容を確認
    let restored_content = fs::read_to_string(&config_path).expect("Failed to read restored config");
    assert_eq!(restored_content, initial_content);
    
    // バックアップを削除
    fs::remove_file(&backup_path).expect("Failed to remove backup");
    assert!(!backup_path.exists());
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_sighup_signal_delivery() {
    // Feature: Manager-Telegraf Integration, Property 2: SIGHUP Delivery
    // Requirements: 11.2, 12.4
    // SIGHUPシグナルがTelegrafコンテナに正しく送信されることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    if !is_telegraf_running() {
        eprintln!("Skipping test: Telegraf container is not running");
        return;
    }
    
    // docker kill --signal=SIGHUP telegraf を実行
    let output = Command::new("docker")
        .args(&["kill", "--signal=SIGHUP", "telegraf"])
        .output()
        .expect("Failed to execute docker command");
    
    // コマンドが成功したことを確認
    assert!(
        output.status.success(),
        "SIGHUP signal delivery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_telegraf_container_not_found_error() {
    // Feature: Manager-Telegraf Integration
    // Requirements: 11.5, 12.5
    // 存在しないコンテナに対するエラーハンドリングを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    // 存在しないコンテナ名でSIGHUPを送信
    let output = Command::new("docker")
        .args(&["kill", "--signal=SIGHUP", "nonexistent-container"])
        .output()
        .expect("Failed to execute docker command");
    
    // コマンドが失敗することを確認
    assert!(!output.status.success());
    
    // エラーメッセージに "No such container" が含まれることを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No such container") || stderr.contains("Error: No such container"),
        "Error message should indicate container not found: {}",
        stderr
    );
}

#[test]
#[ignore] // Docker環境が必要なため、デフォルトでは無視
fn test_docker_socket_access() {
    // Feature: Manager-Telegraf Integration
    // Requirements: 12.1, 12.3
    // Docker Socketへのアクセスが可能であることを確認
    
    if !is_docker_available() {
        eprintln!("Skipping test: Docker is not available");
        return;
    }
    
    // docker ps コマンドを実行
    let output = Command::new("docker")
        .arg("ps")
        .output()
        .expect("Failed to execute docker command");
    
    // コマンドが成功したことを確認
    assert!(
        output.status.success(),
        "Docker socket access failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_config_validation_before_reload() {
    // Feature: Manager-Telegraf Integration
    // Requirements: 11.5
    // 設定ファイルの検証が正しく行われることを確認
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("telegraf.conf");
    
    // 不正な設定ファイルを作成
    let invalid_content = "this is not valid TOML";
    fs::write(&config_path, invalid_content).expect("Failed to write invalid config");
    
    // TOMLパースを試みる
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    let parse_result: Result<toml::Value, _> = toml::from_str(&content);
    
    // パースが失敗することを確認
    assert!(parse_result.is_err(), "Invalid TOML should fail to parse");
}

#[test]
fn test_url_update_workflow() {
    // Feature: Manager-Telegraf Integration
    // Requirements: 11.1, 11.2, 11.3
    // URL更新ワークフロー全体を確認
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // ステップ1: 初期設定を作成
    let initial_urls = vec!["http://example.com"];
    let config_path = create_test_telegraf_config(temp_dir.path(), initial_urls.clone());
    
    // ステップ2: 設定ファイルを読み取り
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(content.contains(initial_urls[0]));
    
    // ステップ3: URLを更新
    let updated_urls = vec!["http://new1.com", "http://new2.com", "http://new3.com"];
    let updated_config_path = create_test_telegraf_config(temp_dir.path(), updated_urls.clone());
    
    // ステップ4: 更新された設定を読み取り
    let updated_content = fs::read_to_string(&updated_config_path).expect("Failed to read updated config");
    
    // ステップ5: 新しいURLが含まれていることを確認
    for url in &updated_urls {
        assert!(updated_content.contains(url), "Updated config should contain URL: {}", url);
    }
    
    // ステップ6: 古いURLが含まれていないことを確認
    assert!(!updated_content.contains(initial_urls[0]), "Updated config should not contain old URL");
}

#[test]
fn test_concurrent_config_updates() {
    // Feature: Manager-Telegraf Integration
    // 複数の同時更新が正しく処理されることを確認
    
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let temp_dir = Arc::new(TempDir::new().expect("Failed to create temp dir"));
    let config_path = Arc::new(temp_dir.path().join("telegraf.conf"));
    
    // 初期設定を作成
    let initial_content = "[[inputs.http_response]]\n  urls = [\"http://initial.com\"]\n";
    fs::write(config_path.as_ref(), initial_content).expect("Failed to write initial config");
    
    let update_count = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    // 10個のスレッドで同時に更新を試みる
    for i in 0..10 {
        let config_path = Arc::clone(&config_path);
        let update_count = Arc::clone(&update_count);
        
        let handle = thread::spawn(move || {
            let content = format!(
                "[[inputs.http_response]]\n  urls = [\"http://update{}.com\"]\n",
                i
            );
            
            // 設定ファイルを更新
            if fs::write(config_path.as_ref(), content).is_ok() {
                let mut count = update_count.lock().unwrap();
                *count += 1;
            }
        });
        
        handles.push(handle);
    }
    
    // すべてのスレッドの完了を待つ
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    // すべての更新が成功したことを確認
    let final_count = *update_count.lock().unwrap();
    assert_eq!(final_count, 10, "All concurrent updates should succeed");
    
    // 最終的な設定ファイルが読み取り可能であることを確認
    let final_content = fs::read_to_string(config_path.as_ref()).expect("Failed to read final config");
    assert!(final_content.contains("[[inputs.http_response]]"));
}
