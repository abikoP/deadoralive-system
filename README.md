# Dead or Alive System

統合Webサイト監視・管理システム

## 概要

Dead or Alive Systemは、Webサイトの死活監視と管理を統合したモノレポ構成のシステムです。Telegraf、InfluxDB、Grafanaによる監視基盤と、Rust製の管理Webアプリケーションを組み合わせ、効率的なWebサイト監視を実現します。

### 主な機能

- **Webサイト死活監視**: Telegrafによる定期的なHTTP/HTTPSエンドポイントチェック
- **メトリクス保存**: InfluxDBによる時系列データの永続化
- **可視化**: Grafanaによるダッシュボード表示
- **管理画面**: Rust製Webアプリケーションによる監視対象URL管理
- **自動リロード**: 設定変更時のTelegraf自動リロード（ダウンタイムゼロ）

## システム構成

```
┌─────────────────────────────────────────────────────────┐
│                    Nginx (Reverse Proxy)                 │
│              doa.com / admin.doa.com                     │
└─────────────────────────────────────────────────────────┘
                    │                    │
        ┌───────────┘                    └───────────┐
        │                                            │
┌───────▼────────┐                        ┌─────────▼──────┐
│    Grafana     │                        │    Manager     │
│  (可視化)       │                        │  (管理画面)     │
└───────┬────────┘                        └─────────┬──────┘
        │                                            │
        │                                    ┌───────▼──────┐
        │                                    │   Telegraf   │
        │                                    │   (監視)      │
        │                                    └───────┬──────┘
        │                                            │
        └────────────────┬───────────────────────────┘
                         │
                  ┌──────▼──────┐
                  │   InfluxDB  │
                  │ (データ保存) │
                  └─────────────┘
```

## クイックスタート

### 前提条件

- Docker & Docker Compose
- Git
- (オプション) Make

### セットアップ手順

1. **リポジトリのクローン**

```bash
git clone <repository-url>
cd deadoralive-system
```

2. **環境変数の設定**

```bash
cp .env.example .env
# .envファイルを編集して、パスワードやシークレットを設定
nano .env
```

3. **システムの起動**

```bash
# Makefileを使用する場合
make up

# または直接Docker Composeを使用
docker-compose up -d
```

4. **アクセス確認**

- Grafana: https://doa.com
- 管理画面: https://admin.doa.com

### 初回ログイン

**管理画面**
- URL: https://admin.doa.com
- ユーザー名: `.env`で設定した値
- パスワード: `.env`で設定した値

**Grafana**
- URL: https://doa.com
- ユーザー名: admin
- パスワード: `.env`で設定した`GRAFANA_ADMIN_PASSWORD`

## ディレクトリ構造

```
deadoralive-system/
├── docker-compose.yml       # Docker Compose設定
├── .env.example             # 環境変数テンプレート
├── .env                     # 環境変数（gitignore）
├── Makefile                 # 運用コマンド
├── README.md                # このファイル
│
├── manager/                 # 管理系（Rust）
│   ├── Dockerfile
│   ├── Cargo.toml
│   ├── src/                 # Rustソースコード
│   ├── config/              # アプリケーション設定
│   └── assets/              # HTMLテンプレート
│
├── monitoring/              # 監視系
│   ├── telegraf/            # Telegraf設定
│   ├── influxdb/            # InfluxDB設定
│   └── grafana/             # Grafana設定
│
├── nginx/                   # リバースプロキシ
│   ├── nginx.conf           # Nginx設定（Let's Encrypt用）
│   ├── nginx-aws.conf       # Nginx設定（AWS ALB用）
│   └── ssl/                 # SSL証明書
│
├── scripts/                 # 運用スクリプト
│   ├── deploy.sh
│   ├── backup.sh
│   ├── restore.sh
│   └── health-check.sh
│
└── docs/                    # ドキュメント
    ├── DEPLOYMENT.md        # デプロイ手順
    ├── OPERATION.md         # 運用手順
    ├── TROUBLESHOOTING.md   # トラブルシューティング
    └── ARCHITECTURE.md      # アーキテクチャ詳細
```

## 基本的な使い方

### URL監視の追加

1. 管理画面（https://admin.doa.com）にログイン
2. 「URL一覧」から「編集」をクリック
3. 監視したいURLを追加
4. 「保存」をクリック
5. Telegrafが自動的に設定をリロード（再起動不要）

### ダッシュボードの確認

1. Grafana（https://doa.com）にアクセス
2. ダッシュボードから監視状況を確認
3. アラート設定やグラフのカスタマイズが可能

## テスト

本プロジェクトでは、包括的なテストスイートを提供しています。

### テストの種類

- **Unit Tests**: 個々のコンポーネントの動作を検証
- **Property-Based Tests**: ランダムな入力で正確性を検証（最小100回実行）
- **Integration Tests**: 複数のコンポーネント間の連携を検証
- **End-to-End Tests**: システム全体の動作を検証

### テストの実行

```bash
# すべてのテストを実行
cd manager
cargo test

# Unit Testsのみ実行
cargo test --lib unit

# Property-Based Testsのみ実行
cargo test --lib property

# Integration Testsのみ実行
cargo test --lib integration

# E2E Testsを実行（Docker Compose環境が必要）
docker compose up -d
cargo test --test e2e_tests -- --ignored
```

### テストカバレッジ

```bash
# カバレッジレポートを生成（cargo-tarpaulinが必要）
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

詳細は[テスト実行ガイド](manager/tests/README.md)を参照してください。

### CI/CD

GitHub Actionsで以下のテストが自動実行されます：

- **プルリクエスト時**: Format Check、Clippy Check、Unit Tests、Property-Based Tests、Integration Tests
- **マージ後**: 上記すべて + E2E Tests

テストが失敗した場合、マージはブロックされます。

## 運用コマンド

### Makefileコマンド

```bash
make up          # 全サービスを起動
make down        # 全サービスを停止
make restart     # 全サービスを再起動
make logs        # ログを表示
make logs-f      # ログをフォロー
make ps          # サービス状態を確認
make build       # イメージをビルド
make clean       # ボリュームを含めて削除
make help        # ヘルプを表示
```

### Docker Composeコマンド

```bash
docker-compose up -d              # バックグラウンドで起動
docker-compose down               # 停止
docker-compose logs -f            # ログをフォロー
docker-compose ps                 # サービス状態確認
docker-compose exec manager sh   # Managerコンテナに入る
```

## ドキュメント

詳細なドキュメントは`docs/`ディレクトリを参照してください：

- **[デプロイ手順](docs/DEPLOYMENT.md)** - 本番環境へのデプロイ方法
- **[運用手順](docs/OPERATION.md)** - 日常的な運用タスク
- **[トラブルシューティング](docs/TROUBLESHOOTING.md)** - よくある問題と解決方法
- **[アーキテクチャ](docs/ARCHITECTURE.md)** - システムアーキテクチャの詳細

## セキュリティ

### 重要な注意事項

1. **環境変数の保護**
   - `.env`ファイルは絶対にコミットしない
   - 本番環境では強力なパスワードを使用
   - `JWT_SECRET`は十分な長さのランダム文字列を使用

2. **SSL/TLS設定**
   - 本番環境では必ずHTTPSを使用
   - Let's EncryptまたはAWS ACMで証明書を取得

3. **アクセス制限**
   - 管理画面へのアクセスをIP制限することを推奨
   - Nginxの設定で`allow`/`deny`ディレクティブを使用

4. **Docker Socket**
   - Managerコンテナのみがアクセス可能
   - 読み取り専用マウントを推奨

## トラブルシューティング

### サービスが起動しない

```bash
# ログを確認
docker-compose logs

# 特定のサービスのログを確認
docker-compose logs manager
docker-compose logs telegraf
```

### Telegrafが設定をリロードしない

```bash
# Telegrafコンテナの状態を確認
docker-compose ps telegraf

# 手動でリロード
docker-compose exec telegraf kill -HUP 1
```

### データベース接続エラー

```bash
# InfluxDBの状態を確認
docker-compose exec influxdb influx ping

# ボリュームを確認
docker volume ls
```

詳細は[トラブルシューティングガイド](docs/TROUBLESHOOTING.md)を参照してください。

## ライセンス

[ライセンス情報を記載]

## 貢献

[貢献ガイドラインを記載]

## サポート

問題が発生した場合は、以下を確認してください：

1. [トラブルシューティングガイド](docs/TROUBLESHOOTING.md)
2. [Issue Tracker](リンクを記載)
3. [ドキュメント](docs/)

## 変更履歴

詳細は[CHANGELOG.md](CHANGELOG.md)を参照してください。
