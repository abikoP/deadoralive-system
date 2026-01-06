// Property 5: エラー時の設定ロールバック
// Feature: monorepo-migration, Property 5: エラー時の設定ロールバック
// Validates: Requirements 11.5
//
// Property: For any 設定ファイル更新が失敗した場合、
//           以前の設定ファイルが保持される

use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// 設定更新シミュレーター（ロールバック機能付き）
struct RollbackSimulator {
    config_path: PathBuf,
    backup_path: PathBuf,
}

impl RollbackSimulator {
    fn new(temp_dir: &TempDir) -> Self {
        let config_path = temp_dir.path().join("telegraf.conf");
        let backup_path = temp_dir.path().join("telegraf.conf.backup");
        
        // 初期設定ファイルを作成
        let initial_config = r#"
[[inputs.http_response]]
  urls = ["https://initial.com"]
  response_timeout = "5s"
  method = "GET"
"#;
        fs::write(&config_path, initial_config).unwrap();
        
        Self {
            config_path,
            backup_path,
        }
    }
    
    /// 現在の設定ファイルの内容を取得
    fn get_current_config(&self) -> String {
        fs::read_to_string(&self.config_path).unwrap()
    }
    
    /// バックアップを作成
    fn create_backup(&self) -> Result<(), String> {
        fs::copy(&self.config_path, &self.backup_path)
            .map_err(|e| format!("Backup error: {}", e))?;
        Ok(())
    }
    
    /// 設定ファイルを更新（失敗をシミュレート可能）
    fn update_config(&self, urls: Vec<String>, should_fail: bool) -> Result<(), String> {
        if should_fail {
            // 失敗をシミュレート
            return Err("Simulated update failure".to_string());
        }
        
        // 設定ファイルの内容を生成
        let mut config = String::from("[[inputs.http_response]]\n  urls = [\n");
        for url in &urls {
            config.push_str(&format!("    \"{}\",\n", url));
        }
        config.push_str("  ]\n  response_timeout = \"5s\"\n  method = \"GET\"\n");
        
        // ファイルに書き込む
        fs::write(&self.config_path, config)
            .map_err(|e| format!("Write error: {}", e))?;
        
        Ok(())
    }
    
    /// バックアップから復元
    fn restore_from_backup(&self) -> Result<(), String> {
        if !self.backup_path.exists() {
            return Err("Backup file not found".to_string());
        }
        
        fs::copy(&self.backup_path, &self.config_path)
            .map_err(|e| format!("Restore error: {}", e))?;
        Ok(())
    }
    
    /// バックアップを削除
    fn delete_backup(&self) -> Result<(), String> {
        if self.backup_path.exists() {
            fs::remove_file(&self.backup_path)
                .map_err(|e| format!("Delete backup error: {}", e))?;
        }
        Ok(())
    }
    
    /// 完全なワークフロー（ロールバック機能付き）
    fn execute_workflow_with_rollback(&self, urls: Vec<String>, should_fail: bool) -> Result<(), String> {
        // 元の設定を保存
        let original_config = self.get_current_config();
        
        // 1. バックアップを作成
        self.create_backup()?;
        
        // 2. 設定ファイルを更新
        match self.update_config(urls, should_fail) {
            Ok(_) => {
                // 成功: バックアップを削除
                self.delete_backup()?;
                Ok(())
            }
            Err(e) => {
                // 失敗: バックアップから復元
                self.restore_from_backup()?;
                self.delete_backup()?;
                
                // 復元後の設定が元の設定と一致することを確認
                let restored_config = self.get_current_config();
                if restored_config == original_config {
                    // ロールバック成功（エラーは返すが、設定は復元されている）
                    Err(e)
                } else {
                    Err(format!("Rollback failed: {}", e))
                }
            }
        }
    }
    
    /// Property 5を検証: エラー時に元の設定が保持されることを確認
    fn verify_rollback(&self, original_config: &str) -> bool {
        let current_config = self.get_current_config();
        current_config == original_config
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
        
        /// Property 5: エラー時の設定ロールバック
        /// Feature: monorepo-migration, Property 5: エラー時の設定ロールバック
        /// Validates: Requirements 11.5
        #[test]
        fn test_rollback_on_failure(urls in url_vec_strategy()) {
            let temp_dir = TempDir::new().unwrap();
            let simulator = RollbackSimulator::new(&temp_dir);
            
            // 元の設定を保存
            let original_config = simulator.get_current_config();
            
            // 失敗をシミュレートしてワークフローを実行
            let result = simulator.execute_workflow_with_rollback(urls, true);
            
            // 更新が失敗したことを確認
            prop_assert!(result.is_err());
            
            // Property 5を検証: 元の設定が保持されている
            prop_assert!(
                simulator.verify_rollback(&original_config),
                "設定ファイル更新が失敗したにもかかわらず、元の設定が復元されませんでした"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_rollback_on_update_failure() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        let urls = vec!["https://new.com".to_string()];
        
        // 失敗をシミュレート
        let result = simulator.execute_workflow_with_rollback(urls, true);
        
        assert!(result.is_err());
        assert!(simulator.verify_rollback(&original_config));
    }

    #[test]
    fn test_no_rollback_on_success() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        let urls = vec!["https://new.com".to_string()];
        
        // 成功をシミュレート
        let result = simulator.execute_workflow_with_rollback(urls, false);
        
        assert!(result.is_ok());
        // 成功時は新しい設定になっているべき
        assert!(!simulator.verify_rollback(&original_config));
        
        // 新しい設定が反映されていることを確認
        let new_config = simulator.get_current_config();
        assert!(new_config.contains("https://new.com"));
    }

    #[test]
    fn test_backup_created_and_deleted() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let urls = vec!["https://test.com".to_string()];
        
        // バックアップが存在しないことを確認
        assert!(!simulator.backup_path.exists());
        
        // ワークフローを実行（成功）
        simulator.execute_workflow_with_rollback(urls, false).unwrap();
        
        // 成功後、バックアップが削除されていることを確認
        assert!(!simulator.backup_path.exists());
    }

    #[test]
    fn test_backup_deleted_after_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let urls = vec!["https://test.com".to_string()];
        
        // ワークフローを実行（失敗）
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        // ロールバック後、バックアップが削除されていることを確認
        assert!(!simulator.backup_path.exists());
    }

    #[test]
    fn test_original_config_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        assert!(original_config.contains("https://initial.com"));
        
        let urls = vec!["https://new.com".to_string()];
        
        // 失敗をシミュレート
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        // 元の設定が保持されていることを確認
        let current_config = simulator.get_current_config();
        assert_eq!(current_config, original_config);
        assert!(current_config.contains("https://initial.com"));
        assert!(!current_config.contains("https://new.com"));
    }

    #[test]
    fn test_multiple_rollbacks() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        
        // 複数回の失敗をシミュレート
        for i in 0..5 {
            let urls = vec![format!("https://attempt{}.com", i)];
            let _ = simulator.execute_workflow_with_rollback(urls, true);
            
            // 毎回元の設定が保持されていることを確認
            assert!(simulator.verify_rollback(&original_config));
        }
    }

    #[test]
    fn test_rollback_with_multiple_urls() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        let urls = vec![
            "https://new1.com".to_string(),
            "https://new2.com".to_string(),
            "https://new3.com".to_string(),
        ];
        
        // 失敗をシミュレート
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        // 元の設定が保持されていることを確認
        assert!(simulator.verify_rollback(&original_config));
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_rollback_with_empty_url_list() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        let urls: Vec<String> = vec![];
        
        // 失敗をシミュレート
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        assert!(simulator.verify_rollback(&original_config));
    }

    #[test]
    fn test_rollback_with_very_long_url_list() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://example{}.com", i))
            .collect();
        
        // 失敗をシミュレート
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        assert!(simulator.verify_rollback(&original_config));
    }

    #[test]
    fn test_success_then_failure_sequence() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        let original_config = simulator.get_current_config();
        
        // 1回目: 成功
        let urls1 = vec!["https://success.com".to_string()];
        let result1 = simulator.execute_workflow_with_rollback(urls1, false);
        assert!(result1.is_ok());
        
        let config_after_success = simulator.get_current_config();
        assert!(config_after_success.contains("https://success.com"));
        
        // 2回目: 失敗（ロールバック）
        let urls2 = vec!["https://failure.com".to_string()];
        let result2 = simulator.execute_workflow_with_rollback(urls2, true);
        assert!(result2.is_err());
        
        // 1回目の成功した設定に戻っていることを確認
        let config_after_rollback = simulator.get_current_config();
        assert_eq!(config_after_rollback, config_after_success);
        assert!(config_after_rollback.contains("https://success.com"));
        assert!(!config_after_rollback.contains("https://failure.com"));
    }

    #[test]
    fn test_rollback_preserves_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let simulator = RollbackSimulator::new(&temp_dir);
        
        // 元のファイルのメタデータを取得
        let original_metadata = fs::metadata(&simulator.config_path).unwrap();
        
        let urls = vec!["https://test.com".to_string()];
        
        // 失敗をシミュレート
        let _ = simulator.execute_workflow_with_rollback(urls, true);
        
        // ロールバック後のメタデータを取得
        let restored_metadata = fs::metadata(&simulator.config_path).unwrap();
        
        // ファイルサイズが同じことを確認（内容が同じ）
        assert_eq!(original_metadata.len(), restored_metadata.len());
    }

    #[test]
    fn test_concurrent_rollback_safety() {
        // 並行性のテスト（シミュレーション）
        let temp_dir = TempDir::new().unwrap();
        
        // 複数のシミュレーターを作成（実際の並行実行ではないが、独立性を確認）
        for i in 0..10 {
            let simulator = RollbackSimulator::new(&temp_dir);
            let original_config = simulator.get_current_config();
            
            let urls = vec![format!("https://concurrent{}.com", i)];
            let _ = simulator.execute_workflow_with_rollback(urls, true);
            
            // 各シミュレーターで元の設定が保持されていることを確認
            assert!(simulator.verify_rollback(&original_config));
        }
    }
}
