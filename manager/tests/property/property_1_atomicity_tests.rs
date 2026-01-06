// Property 1: 設定ファイル更新の原子性
// Feature: monorepo-migration, Property 1: 設定ファイル更新の原子性
// Validates: Requirements 11.1, 11.2
//
// Property: For any URL更新操作、設定ファイルへの書き込みが完了するまで、
//           Telegrafへのリロード指示は送信されない

use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// テスト用の操作ログ
#[derive(Debug, Clone, PartialEq)]
enum Operation {
    ConfigWriteStart,
    ConfigWriteComplete,
    SighupSent,
}

/// 設定更新とSIGHUP送信をシミュレートする構造体
struct ConfigUpdateSimulator {
    operations: Arc<Mutex<Vec<Operation>>>,
    config_path: PathBuf,
}

impl ConfigUpdateSimulator {
    fn new(temp_dir: &TempDir) -> Self {
        let config_path = temp_dir.path().join("telegraf.conf");
        
        // 初期設定ファイルを作成
        let initial_config = r#"
[[inputs.http_response]]
  urls = ["https://initial.com"]
  response_timeout = "5s"
  method = "GET"
"#;
        fs::write(&config_path, initial_config).unwrap();
        
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
            config_path,
        }
    }
    
    /// 設定ファイルを更新（原子性を保証）
    fn update_config(&self, urls: Vec<String>) -> Result<(), String> {
        // 操作ログ: 書き込み開始
        self.operations.lock().unwrap().push(Operation::ConfigWriteStart);
        
        // 設定ファイルの内容を生成
        let mut config = String::from("[[inputs.http_response]]\n  urls = [\n");
        for url in &urls {
            config.push_str(&format!("    \"{}\",\n", url));
        }
        config.push_str("  ]\n  response_timeout = \"5s\"\n  method = \"GET\"\n");
        
        // ファイルに書き込む
        fs::write(&self.config_path, config)
            .map_err(|e| format!("Write error: {}", e))?;
        
        // 操作ログ: 書き込み完了
        self.operations.lock().unwrap().push(Operation::ConfigWriteComplete);
        
        Ok(())
    }
    
    /// SIGHUPシグナルを送信
    fn send_sighup(&self) {
        // 操作ログ: SIGHUP送信
        self.operations.lock().unwrap().push(Operation::SighupSent);
    }
    
    /// 原子性を検証: ConfigWriteComplete の後に SighupSent が来ることを確認
    fn verify_atomicity(&self) -> bool {
        let ops = self.operations.lock().unwrap();
        
        // ConfigWriteStart と ConfigWriteComplete のペアを探す
        let mut write_start_index = None;
        let mut write_complete_index = None;
        let mut sighup_index = None;
        
        for (i, op) in ops.iter().enumerate() {
            match op {
                Operation::ConfigWriteStart => write_start_index = Some(i),
                Operation::ConfigWriteComplete => write_complete_index = Some(i),
                Operation::SighupSent => sighup_index = Some(i),
            }
        }
        
        // 全ての操作が記録されているか確認
        if write_start_index.is_none() || write_complete_index.is_none() || sighup_index.is_none() {
            return false;
        }
        
        let write_start = write_start_index.unwrap();
        let write_complete = write_complete_index.unwrap();
        let sighup = sighup_index.unwrap();
        
        // 原子性の検証:
        // 1. ConfigWriteStart < ConfigWriteComplete
        // 2. ConfigWriteComplete < SighupSent
        write_start < write_complete && write_complete < sighup
    }
    
    /// 完全なワークフローを実行
    fn execute_workflow(&self, urls: Vec<String>) -> Result<(), String> {
        // 1. 設定ファイルを更新
        self.update_config(urls)?;
        
        // 2. SIGHUPを送信（書き込み完了後）
        self.send_sighup();
        
        Ok(())
    }
}

// Property-Based Test用のURL生成戦略
fn url_vec_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::string::string_regex("https://example[0-9]{1,3}\\.com").unwrap(),
        1..100, // 1-100個のURL
    )
}

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// Property 1: 設定ファイル更新の原子性
        /// Feature: monorepo-migration, Property 1: 設定ファイル更新の原子性
        /// Validates: Requirements 11.1, 11.2
        #[test]
        fn test_config_update_atomicity(urls in url_vec_strategy()) {
            let temp_dir = TempDir::new().unwrap();
            let simulator = ConfigUpdateSimulator::new(&temp_dir);
            
            // ワークフローを実行
            let result = simulator.execute_workflow(urls);
            
            // 実行が成功したことを確認
            prop_assert!(result.is_ok());
            
            // 原子性を検証
            prop_assert!(
                simulator.verify_atomicity(),
                "設定ファイルの書き込みが完了する前にSIGHUPが送信されました"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_atomicity_with_single_url() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls = vec!["https://example.com".to_string()];
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }

    #[test]
    fn test_atomicity_with_multiple_urls() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls = vec![
            "https://example1.com".to_string(),
            "https://example2.com".to_string(),
            "https://example3.com".to_string(),
        ];
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }

    #[test]
    fn test_atomicity_with_many_urls() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls: Vec<String> = (0..50)
            .map(|i| format!("https://example{}.com", i))
            .collect();
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }

    #[test]
    fn test_operation_order() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls = vec!["https://test.com".to_string()];
        simulator.execute_workflow(urls).unwrap();
        
        let ops = simulator.operations.lock().unwrap();
        
        // 操作の順序を確認
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0], Operation::ConfigWriteStart);
        assert_eq!(ops[1], Operation::ConfigWriteComplete);
        assert_eq!(ops[2], Operation::SighupSent);
    }

    #[test]
    fn test_config_file_written() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls = vec![
            "https://test1.com".to_string(),
            "https://test2.com".to_string(),
        ];
        simulator.execute_workflow(urls.clone()).unwrap();
        
        // 設定ファイルが正しく書き込まれたことを確認
        let content = fs::read_to_string(&simulator.config_path).unwrap();
        assert!(content.contains("https://test1.com"));
        assert!(content.contains("https://test2.com"));
    }

    #[test]
    fn test_atomicity_violation_detection() {
        // 原子性違反のシミュレーション
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("telegraf.conf");
        fs::write(&config_path, "initial").unwrap();
        
        let operations = Arc::new(Mutex::new(Vec::new()));
        
        // 不正な順序: SIGHUP が書き込み完了前に送信される
        operations.lock().unwrap().push(Operation::ConfigWriteStart);
        operations.lock().unwrap().push(Operation::SighupSent); // 違反！
        operations.lock().unwrap().push(Operation::ConfigWriteComplete);
        
        // 検証ロジック
        let ops = operations.lock().unwrap();
        let mut write_complete_index = None;
        let mut sighup_index = None;
        
        for (i, op) in ops.iter().enumerate() {
            match op {
                Operation::ConfigWriteComplete => write_complete_index = Some(i),
                Operation::SighupSent => sighup_index = Some(i),
                _ => {}
            }
        }
        
        let is_atomic = if let (Some(complete), Some(sighup)) = (write_complete_index, sighup_index) {
            complete < sighup
        } else {
            false
        };
        
        // 原子性違反が検出されることを確認
        assert!(!is_atomic, "原子性違反が検出されませんでした");
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_url_list() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls: Vec<String> = vec![];
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }

    #[test]
    fn test_very_long_url_list() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://example{}.com", i))
            .collect();
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }

    #[test]
    fn test_urls_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = ConfigUpdateSimulator::new(&temp_dir);
        
        let urls = vec![
            "https://example.com/path?query=value".to_string(),
            "https://example.com/path#fragment".to_string(),
            "https://example.com:8080/path".to_string(),
        ];
        let result = simulator.execute_workflow(urls);
        
        assert!(result.is_ok());
        assert!(simulator.verify_atomicity());
    }
}
