# Implementation Plan

- [x] 1. 新規リポジトリの準備
  - 親ディレクトリに`deadoralive-system`ディレクトリを作成
  - Git初期化
  - 基本的なディレクトリ構造を作成
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. 基本設定ファイルの作成
  - `.gitignore`を作成
  - `.env.example`を作成
  - `README.md`を作成（システム概要）
  - _Requirements: 7.1, 7.3, 8.1, 8.2, 13.1_

- [x] 3. 管理系（Manager）の移行
  - `manager/`ディレクトリを作成
  - Rustソースコードを移行（`src/`）
  - 設定ファイルを移行（`config/`）
  - アセットを移行（`assets/`）
  - `Cargo.toml`と`Cargo.lock`を移行
  - `migration/`ディレクトリを移行
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 4. Manager用Dockerfileの作成
  - `manager/Dockerfile`を作成（マルチステージビルド）
  - `.dockerignore`を作成
  - Docker CLIをインストール（Telegraf制御用）
  - _Requirements: 2.4, 12.2_

- [x] 5. 監視系（Monitoring）の移行
  - `monitoring/`ディレクトリを作成
  - Telegraf設定を移行（`monitoring/telegraf/`）
  - InfluxDB設定を移行（`monitoring/influxdb/`）
  - Grafana設定を移行（`monitoring/grafana/`）
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 6. Nginx設定の作成
  - `nginx/`ディレクトリを作成
  - `nginx/nginx.conf`を作成（Let's Encrypt用・HTTPS対応）
  - `nginx/nginx-aws.conf`を作成（AWS ALB + ACM用・HTTP のみ）
  - SSL証明書ディレクトリを作成（`nginx/ssl/`）
  - ドメインルーティング設定（doa.com, admin.doa.com）
  - セキュリティヘッダー設定
  - Rate Limiting設定
  - パフォーマンス最適化（Gzip, HTTP/2, バッファリング）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 7. Docker Compose統合設定の作成
  - `docker-compose.yml`を作成
  - Nginxサービスを定義
  - Managerサービスを定義
  - Telegrafサービスを定義
  - InfluxDBサービスを定義
  - Grafanaサービスを定義
  - _Requirements: 4.1, 4.2_

- [x] 8. Docker Composeボリューム設定
  - `telegraf-config`ボリュームを定義（設定ファイル共有）
  - `influxdb-data`ボリュームを定義（データ永続化）
  - `grafana-data`ボリュームを定義（ダッシュボード永続化）
  - `manager-data`ボリュームを定義（SQLite DB）
  - _Requirements: 4.3_

- [x] 9. Docker Composeネットワーク設定
  - `frontend`ネットワークを定義
  - `backend`ネットワークを定義
  - 各サービスを適切なネットワークに配置
  - _Requirements: 4.4_

- [x] 10. Docker Composeヘルスチェック設定
  - Nginxのヘルスチェックを定義
  - Managerのヘルスチェックを定義
  - InfluxDBのヘルスチェックを定義
  - Grafanaのヘルスチェックを定義
  - _Requirements: 4.5_

- [x] 11. Manager: Docker Socket統合
  - `docker-compose.yml`でDocker Socketをマウント
  - Managerコンテナに読み取り専用でマウント
  - 環境変数`TELEGRAF_CONTAINER_NAME`を設定
  - _Requirements: 12.1, 12.3_

- [x] 12. Manager: Telegraf制御機能の実装
  - `src/controllers/telegraf.rs`を更新
  - `reload`関数を実装（SIGHUPシグナル送信）
  - Docker CLIを使用してシグナル送信
  - エラーハンドリングを実装
  - _Requirements: 11.2, 11.3, 11.4, 11.5, 12.4, 12.5_

- [x] 13. Manager: 設定ファイル更新機能の強化
  - `src/services/config_service.rs`を更新
  - 設定ファイル更新前のバックアップ機能
  - 更新失敗時のロールバック機能
  - 共有ボリュームへの書き込み
  - _Requirements: 11.1_

- [x] 14. Makefileの作成
  - `Makefile`を作成
  - `make up`コマンド（全サービス起動）
  - `make down`コマンド（全サービス停止）
  - `make logs`コマンド（ログ表示）
  - `make help`コマンド（ヘルプ表示）
  - その他の運用コマンド
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 15. 運用スクリプトの作成
  - `scripts/`ディレクトリを作成
  - `scripts/deploy.sh`を作成
  - `scripts/backup.sh`を作成
  - `scripts/restore.sh`を作成
  - `scripts/health-check.sh`を作成
  - `scripts/ssl-renew.sh`を作成
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [x] 16. ドキュメントの作成
  - `docs/`ディレクトリを作成
  - `docs/DEPLOYMENT.md`を作成
  - `docs/OPERATION.md`を作成
  - `docs/TROUBLESHOOTING.md`を作成
  - `docs/ARCHITECTURE.md`を作成
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 17. README.mdの完成
  - クイックスタート手順を追加
  - ディレクトリ構造の説明を追加
  - デプロイ手順へのリンクを追加
  - トラブルシューティングへのリンクを追加
  - _Requirements: 8.3, 8.4, 8.5_

- [x] 18. 環境変数設定の完成
  - `.env.example`にJWT_SECRETを追加
  - `.env.example`にINFLUXDB設定を追加
  - `.env.example`にGRAFANA設定を追加
  - `.gitignore`に`.env`を追加
  - _Requirements: 7.2, 7.4, 13.2_

- [x] 19. ビルドテスト
  - `make build`を実行
  - 全てのDockerイメージがビルドされることを確認
  - ビルドエラーがないことを確認
  - _Requirements: 4.1_

- [x] 20. 起動テスト
  - `make up`を実行
  - 全てのサービスが起動することを確認
  - ヘルスチェックが成功することを確認
  - _Requirements: 4.1, 4.5_

- [x] 21. 統合テスト
  - 管理画面にアクセスできることを確認
  - URLを更新できることを確認
  - Telegraf設定がリロードされることを確認
  - Grafanaにアクセスできることを確認
  - _Requirements: 11.1, 11.2, 11.3_

- [x] 22. Git初期化とコミット
  - `git add .`を実行
  - 初回コミット
  - リモートリポジトリを追加（オプション）
  - プッシュ（オプション）
  - _Requirements: 1.1_

- [x] 23. 最終確認とドキュメント更新
  - 全ての機能が動作することを確認
  - ドキュメントの最終レビュー
  - README.mdの最終更新
  - _Requirements: 8.1_
