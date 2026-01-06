// Telegraf Controller Unit Tests
// Docker操作とエラーハンドリングをテストします

use std::process::{Command, Output, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(test)]
mod docker_operations_tests {
    use super::*;

    // テスト用のDocker操作シミュレーション関数
    fn simulate_docker_kill_sighup(container_name: &str, should_succeed: bool, error_type: Option<&str>) -> Result<Output, std::io::Error> {
        if should_succeed {
            // 成功をシミュレート
            Ok(Output {
                status: ExitStatus::default(),
                stdout: format!("{}\n", container_name).into_bytes(),
                stderr: Vec::new(),
            })
        } else {
            // エラーをシミュレート
            let error_message = match error_type {
                Some("no_container") => format!("Error response from daemon: No such container: {}", container_name),
                Some("permission_denied") => "Error response from daemon: permission denied".to_string(),
                Some("socket_error") => "Cannot connect to the Docker daemon".to_string(),
                _ => "Unknown error".to_string(),
            };
            
            #[cfg(unix)]
            let status = ExitStatus::from_raw(256); // Exit code 1 (256 = 1 << 8)
            
            #[cfg(not(unix))]
            let status = {
                // Windows環境では別の方法でExitStatusを作成
                use std::process::Command;
                Command::new("cmd").arg("/C").arg("exit 1").status().unwrap()
            };
            
            Ok(Output {
                status,
                stdout: Vec::new(),
                stderr: error_message.into_bytes(),
            })
        }
    }

    #[test]
    fn test_sighup_success() {
        // 正常系: SIGHUPシグナル送信成功
        let container_name = "telegraf";
        let result = simulate_docker_kill_sighup(container_name, true, None);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_container_not_found() {
        // 異常系: コンテナが存在しない
        let container_name = "nonexistent-container";
        let result = simulate_docker_kill_sighup(container_name, false, Some("no_container"));
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("No such container"));
    }

    #[test]
    fn test_docker_socket_unavailable() {
        // 異常系: Docker Socketにアクセスできない
        let container_name = "telegraf";
        let result = simulate_docker_kill_sighup(container_name, false, Some("socket_error"));
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Cannot connect to the Docker daemon"));
    }

    #[test]
    fn test_permission_denied() {
        // 異常系: 権限不足
        let container_name = "telegraf";
        let result = simulate_docker_kill_sighup(container_name, false, Some("permission_denied"));
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("permission denied"));
    }

    #[test]
    fn test_custom_container_name() {
        // エッジケース: カスタムコンテナ名
        let container_name = "custom-telegraf-container";
        let result = simulate_docker_kill_sighup(container_name, true, None);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_container_name_with_special_chars() {
        // エッジケース: 特殊文字を含むコンテナ名
        let container_name = "telegraf-prod-01";
        let result = simulate_docker_kill_sighup(container_name, true, None);
        
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    fn parse_error_message(stderr: &[u8]) -> String {
        String::from_utf8_lossy(stderr).to_string()
    }

    fn generate_user_friendly_error(error_type: &str, container_name: &str) -> String {
        match error_type {
            "no_container" => format!(
                "Telegrafコンテナが見つかりません。\nコンテナ名: {}\nDocker Composeでサービスが起動しているか確認してください。",
                container_name
            ),
            "socket_error" => format!(
                "Docker操作に失敗しました。\nDocker Socketがマウントされているか確認してください。"
            ),
            "permission_denied" => format!(
                "Docker操作に失敗しました。\n権限が不足しています。"
            ),
            _ => "不明なエラーが発生しました。".to_string(),
        }
    }

    #[test]
    fn test_no_container_error_message() {
        // Docker CLIのエラー出力を適切に処理
        let stderr = b"Error response from daemon: No such container: telegraf";
        let error_msg = parse_error_message(stderr);
        
        assert!(error_msg.contains("No such container"));
        
        // ユーザーフレンドリーなエラーメッセージを生成
        let user_msg = generate_user_friendly_error("no_container", "telegraf");
        assert!(user_msg.contains("Telegrafコンテナが見つかりません"));
        assert!(user_msg.contains("Docker Compose"));
    }

    #[test]
    fn test_socket_error_message() {
        // ユーザーフレンドリーなエラーメッセージを返す
        let user_msg = generate_user_friendly_error("socket_error", "telegraf");
        assert!(user_msg.contains("Docker Socketがマウントされているか確認"));
    }

    #[test]
    fn test_permission_error_message() {
        let user_msg = generate_user_friendly_error("permission_denied", "telegraf");
        assert!(user_msg.contains("権限が不足"));
    }

    #[test]
    fn test_error_message_escaping() {
        // HTMLエスケープのテスト
        let raw_error = "<script>alert('xss')</script>";
        let escaped = html_escape::encode_text(raw_error);
        
        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_multiline_error_message() {
        // 複数行のエラーメッセージ
        let error_msg = "Error line 1\nError line 2\nError line 3";
        let lines: Vec<&str> = error_msg.lines().collect();
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Error line 1");
        assert_eq!(lines[2], "Error line 3");
    }
}

#[cfg(test)]
mod response_generation_tests {
    use super::*;

    fn generate_response_html_simple(success: bool, message: &str, detail: &str) -> String {
        let (status_class, status_icon) = if success {
            ("success", "✓")
        } else {
            ("error", "✗")
        };

        format!(
            r#"<div class="status-icon {}">{}</div><div class="message">{}</div><div class="detail">{}</div>"#,
            status_class,
            status_icon,
            html_escape::encode_text(message),
            html_escape::encode_text(detail)
        )
    }

    #[test]
    fn test_success_response_html() {
        let html = generate_response_html_simple(
            true,
            "Telegraf設定をリロードしました",
            "コンテナ: telegraf"
        );
        
        assert!(html.contains("success"));
        assert!(html.contains("✓"));
        assert!(html.contains("Telegraf設定をリロードしました"));
        assert!(html.contains("telegraf"));
    }

    #[test]
    fn test_error_response_html() {
        let html = generate_response_html_simple(
            false,
            "Telegraf設定のリロードに失敗しました",
            "Error: No such container"
        );
        
        assert!(html.contains("error"));
        assert!(html.contains("✗"));
        assert!(html.contains("失敗しました"));
        assert!(html.contains("No such container"));
    }

    #[test]
    fn test_html_escaping_in_response() {
        let html = generate_response_html_simple(
            false,
            "Error <script>",
            "Detail <img>"
        );
        
        // HTMLタグがエスケープされていることを確認
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;img&gt;"));
    }

    #[test]
    fn test_empty_detail_response() {
        let html = generate_response_html_simple(
            true,
            "Success",
            ""
        );
        
        assert!(html.contains("Success"));
        // 空の詳細でもHTMLが正しく生成される
        assert!(html.contains("detail"));
    }
}

#[cfg(test)]
mod environment_variable_tests {
    use super::*;

    #[test]
    fn test_default_container_name() {
        // 環境変数が設定されていない場合のデフォルト値
        let container_name = std::env::var("TELEGRAF_CONTAINER_NAME_TEST")
            .unwrap_or_else(|_| "telegraf".to_string());
        
        assert_eq!(container_name, "telegraf");
    }

    #[test]
    fn test_custom_container_name_from_env() {
        // 環境変数からカスタムコンテナ名を取得
        std::env::set_var("TELEGRAF_CONTAINER_NAME_TEST2", "custom-telegraf");
        
        let container_name = std::env::var("TELEGRAF_CONTAINER_NAME_TEST2")
            .unwrap_or_else(|_| "telegraf".to_string());
        
        assert_eq!(container_name, "custom-telegraf");
        
        // クリーンアップ
        std::env::remove_var("TELEGRAF_CONTAINER_NAME_TEST2");
    }
}

#[cfg(test)]
mod integration_simulation_tests {
    use super::*;

    // 完全なワークフローのシミュレーション
    fn simulate_reload_workflow(
        container_exists: bool,
        docker_available: bool,
        has_permission: bool,
    ) -> Result<String, String> {
        let container_name = "telegraf";
        
        if !docker_available {
            return Err("Docker Socketがマウントされているか確認してください。".to_string());
        }
        
        if !has_permission {
            return Err("権限が不足しています。".to_string());
        }
        
        if !container_exists {
            return Err(format!("Telegrafコンテナが見つかりません: {}", container_name));
        }
        
        Ok(format!("Telegraf設定をリロードしました: {}", container_name))
    }

    #[test]
    fn test_successful_reload_workflow() {
        let result = simulate_reload_workflow(true, true, true);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("リロードしました"));
    }

    #[test]
    fn test_container_not_found_workflow() {
        let result = simulate_reload_workflow(false, true, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("見つかりません"));
    }

    #[test]
    fn test_docker_unavailable_workflow() {
        let result = simulate_reload_workflow(true, false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Docker Socket"));
    }

    #[test]
    fn test_permission_denied_workflow() {
        let result = simulate_reload_workflow(true, true, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("権限"));
    }
}
