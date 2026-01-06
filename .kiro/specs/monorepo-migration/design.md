# Design Document

## Overview

`deadoralive-system`モノレポは、Webサイト監視システム（Telegraf + InfluxDB + Grafana）と、その管理システム（Rust製Webアプリケーション）を統合したプラットフォームです。全てのコンポーネントをDockerコンテナとして実行し、単一のdocker-compose.ymlで管理します。

## Architecture

### システム構成図

```
┌─────────────────────────────────────────────────────────┐
│ deadoralive-system/ (Monorepo Root)                     │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │ Docker Compose Stack                           │    │
│  │                                                 │    │
│  │  ┌──────────────┐  ┌──────────────┐           │    │
│  │  │   Nginx      │  │   Manager    │           │    │
│  │  │  (Proxy)     │  │   (Rust)     │           │    │
│  │  └──────────────┘  └──────────────┘           │    │
│  │         │                  │                    │    │
│  │         │                  │ (Docker Socket)    │    │
│  │         │                  ↓                    │    │
│  │  ┌──────────────┐  ┌──────────────┐           │    │
│  │  │   Grafana    │  │   Telegraf   │←─SIGHUP   │    │
│  │  └──────────────┘  └──────────────┘           │    │
│  │         │                  │                    │    │
│  │         └────→ ┌──────────────┐                │    │
│  │                │   InfluxDB   │                │    │
│  │                └──────────────┘                │    │
│  └────────────────────────────────────────────────┘    │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │ Shared Volumes                                 │    │
│  │  - telegraf-config/ (設定ファイル共有)         │    │
│  │  - influxdb-data/ (データ永続化)               │    │
│  │  - grafana-data/ (ダッシュボード永続化)        │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### ディレクトリ構造

```
deadoralive-system/
├── docker-compose.yml           # 統合オーケストレーション
├── .env.example                 # 環境変数テンプレート
├── .env                         # 環境変数（gitignore）
├── .gitignore                   # Git除外設定
├── Makefile                     # 運用コマンド
├── README.md                    # システム概要
│
├── manager/                     # 管理系（旧kiro_test）
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── Dockerfile               # マルチステージビルド
│   ├── .dockerignore
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── app.rs
│   │   ├── controllers/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── config.rs
│   │   │   ├── admin_list.rs
│   │   │   ├── admin_edit.rs
│   │   │   └── telegraf.rs      # SIGHUP送信機能
│   │   ├── models/
│   │   │   └── mod.rs
│   │   └── services/
│   │       ├── mod.rs
│   │       ├── config_service.rs
│   │       └── url_validation_service.rs
│   ├── config/
│   │   ├── development.yaml
│   │   └── production.yaml
│   ├── assets/
│   │   └── views/
│   ├── migration/
│   └── README.md
│
├── monitoring/                  # 監視系（旧deadoralive）
│   ├── telegraf/
│   │   ├── telegraf.conf        # 共有ボリュームにマウント
│   │   └── telegraf.d/
│   ├── influxdb/
│   │   ├── init-scripts/
│   │   └── influxdb.conf
│   └── grafana/
│       ├── dashboards/
│       ├── datasources/
│       └── grafana.ini
│
├── nginx/                       # リバースプロキシ
│   ├── nginx.conf
│   ├── ssl/                     # SSL証明書（gitignore）
│   └── Dockerfile               # カスタムNginx（オプション）
│
├── scripts/                     # 運用スクリプト
│   ├── deploy.sh
│   ├── backup.sh
│   ├── restore.sh
│   ├── ssl-renew.sh
│   └── health-check.sh
│
└── docs/                        # ドキュメント
    ├── DEPLOYMENT.md
    ├── OPERATION.md
    ├── TROUBLESHOOTING.md
    └── ARCHITECTURE.md
```

## Components and Interfaces

### 1. Manager Service (管理系)

**責務:**
- Telegraf設定ファイルの読み書き
- URL一覧の表示・編集
- ユーザー認証
- Telegrafコンテナへの設定リロード指示

**インターフェース:**
- HTTP API (Port 8081)
  - `GET /auth/login` - ログイン画面
  - `POST /auth/login` - ログイン処理
  - `GET /admin` - 管理画面トップ
  - `GET /admin/list` - URL一覧
  - `GET /admin/edit` - URL編集画面
  - `POST /admin/edit` - URL更新
  - `GET /admin/reload` - Telegraf設定リロード（SIGHUP）

**依存関係:**
- 共有ボリューム: `telegraf-config`（読み書き）
- Docker Socket: `/var/run/docker.sock`（Telegraf制御用）
- SQLite: `manager-data`（ユーザーDB）

### 2. Telegraf Service (監視系)

**責務:**
- HTTP/HTTPSエンドポイントの監視
- メトリクスの収集
- InfluxDBへのデータ送信

**インターフェース:**
- 設定ファイル: `/etc/telegraf/telegraf.conf`（共有ボリュームから読み取り）
- シグナル: SIGHUP（設定リロード）

**依存関係:**
- 共有ボリューム: `telegraf-config`（読み取り専用）
- InfluxDB: データ送信先

### 3. InfluxDB Service

**責務:**
- 時系列データの保存
- クエリ処理

**インターフェース:**
- HTTP API (Port 8086)

**依存関係:**
- 永続ボリューム: `influxdb-data`

### 4. Grafana Service

**責務:**
- ダッシュボード表示
- データ可視化

**インターフェース:**
- HTTP UI (Port 3000)

**依存関係:**
- InfluxDB: データソース
- 永続ボリューム: `grafana-data`

### 5. Nginx Service

**責務:**
- リバースプロキシ
- SSL/TLS終端
- ドメインベースルーティング
- Rate Limiting（レート制限）
- セキュリティヘッダーの付与

**インターフェース:**
- HTTP (Port 80) → HTTPS リダイレクト
- HTTPS (Port 443)
  - `doa.com` → Grafana (Port 3000)
  - `admin.doa.com` → Manager (Port 8081)

**設定詳細:**

**パターンA: ALB + ACM使用（推奨 - AWS環境）**

ALBでSSL/TLS終端を行う場合、NginxはHTTPのみで動作します。

```nginx
events {
    worker_connections 1024;
}

http {
    # ログ設定
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Gzip圧縮
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
    gzip_min_length 1000;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=admin:10m rate=5r/s;

    # Real IP設定（ALB経由の場合）
    set_real_ip_from 10.0.0.0/8;  # VPC CIDR
    set_real_ip_from 172.16.0.0/12;
    real_ip_header X-Forwarded-For;
    real_ip_recursive on;

    # ========================================
    # Grafana (doa.com)
    # ========================================
    server {
        listen 80;
        server_name doa.com;

        # セキュリティヘッダー
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;

        # Rate Limiting
        limit_req zone=general burst=20 nodelay;

        location / {
            proxy_pass http://grafana:3000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $http_x_forwarded_proto;

            # WebSocket対応（Grafanaのライブ更新用）
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";

            # タイムアウト設定
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
    }

    # ========================================
    # Telegraf Manager (admin.doa.com)
    # ========================================
    server {
        listen 80;
        server_name admin.doa.com;

        # セキュリティヘッダー（管理画面は厳しめ）
        add_header X-Frame-Options "DENY" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
        add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;

        # Rate Limiting（管理画面は厳しめ）
        limit_req zone=admin burst=10 nodelay;

        # IP制限（オプション - 必要に応じてコメント解除）
        # allow 203.0.113.0/24;  # 許可するIPレンジ
        # deny all;

        location / {
            proxy_pass http://manager:8081;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $http_x_forwarded_proto;

            # タイムアウト設定
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;

            # バッファ設定
            proxy_buffering on;
            proxy_buffer_size 4k;
            proxy_buffers 8 4k;
        }
    }

    # ========================================
    # ヘルスチェックエンドポイント
    # ========================================
    server {
        listen 80 default_server;
        server_name _;

        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
```

**パターンB: Let's Encrypt使用（EC2単体環境）**

EC2単体で運用する場合、NginxでSSL/TLS終端を行います。

```nginx
events {
    worker_connections 1024;
}

http {
    # ログ設定
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # SSL設定
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256';
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Gzip圧縮
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
    gzip_min_length 1000;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=admin:10m rate=5r/s;

    # ========================================
    # HTTP → HTTPS リダイレクト
    # ========================================
    server {
        listen 80;
        server_name doa.com admin.doa.com;

        # Let's Encrypt検証用
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }

        # その他は全てHTTPSへリダイレクト
        location / {
            return 301 https://$host$request_uri;
        }
    }

    # ========================================
    # Grafana (doa.com)
    # ========================================
    server {
        listen 443 ssl http2;
        server_name doa.com;

        # SSL証明書
        ssl_certificate /etc/nginx/ssl/live/doa.com/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/live/doa.com/privkey.pem;

        # セキュリティヘッダー
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;

        # Rate Limiting
        limit_req zone=general burst=20 nodelay;

        location / {
            proxy_pass http://grafana:3000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # WebSocket対応（Grafanaのライブ更新用）
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";

            # タイムアウト設定
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
    }

    # ========================================
    # Telegraf Manager (admin.doa.com)
    # ========================================
    server {
        listen 443 ssl http2;
        server_name admin.doa.com;

        # SSL証明書
        ssl_certificate /etc/nginx/ssl/live/admin.doa.com/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/live/admin.doa.com/privkey.pem;

        # セキュリティヘッダー（管理画面は厳しめ）
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        add_header X-Frame-Options "DENY" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
        add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;

        # Rate Limiting（管理画面は厳しめ）
        limit_req zone=admin burst=10 nodelay;

        # IP制限（オプション - 必要に応じてコメント解除）
        # allow 203.0.113.0/24;  # 許可するIPレンジ
        # deny all;

        location / {
            proxy_pass http://manager:8081;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # タイムアウト設定
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;

            # バッファ設定
            proxy_buffering on;
            proxy_buffer_size 4k;
            proxy_buffers 8 4k;
        }
    }

    # ========================================
    # ヘルスチェックエンドポイント
    # ========================================
    server {
        listen 80;
        server_name localhost;

        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
```

**セキュリティ機能:**

**パターンA（ALB + ACM）の場合:**

1. **Real IP取得**
   - ALB経由のため、X-Forwarded-Forヘッダーから実IPを取得
   - VPC CIDRからのアクセスを信頼

2. **セキュリティヘッダー**
   - X-Frame-Options: クリックジャッキング防止
   - X-Content-Type-Options: MIMEスニッフィング防止
   - CSP: XSS攻撃防止（管理画面のみ）
   - 注: HSTSはALBで設定

3. **Rate Limiting**
   - 一般ユーザー: 10リクエスト/秒
   - 管理画面: 5リクエスト/秒
   - バースト許容: 一般20、管理10

4. **IP制限（オプション）**
   - 管理画面へのアクセスを特定IPに制限可能

**パターンB（Let's Encrypt）の場合:**

1. **SSL/TLS設定**
   - TLS 1.2/1.3のみ許可
   - 強力な暗号スイート
   - セッションキャッシュで性能向上

2. **セキュリティヘッダー**
   - HSTS: HTTPS強制
   - X-Frame-Options: クリックジャッキング防止
   - X-Content-Type-Options: MIMEスニッフィング防止
   - CSP: XSS攻撃防止（管理画面のみ）

3. **Rate Limiting**
   - 一般ユーザー: 10リクエスト/秒
   - 管理画面: 5リクエスト/秒
   - バースト許容: 一般20、管理10

4. **IP制限（オプション）**
   - 管理画面へのアクセスを特定IPに制限可能

**パフォーマンス最適化:**

1. **Gzip圧縮**
   - テキストベースのコンテンツを圧縮
   - 1KB以上のファイルのみ圧縮

2. **HTTP/2**
   - 多重化による高速化
   - ヘッダー圧縮

3. **プロキシバッファリング**
   - バックエンドの負荷軽減
   - レスポンス時間の改善

**依存関係:**
- Grafana Service: プロキシ先
- Manager Service: プロキシ先
- SSL証明書: 
  - パターンA: AWS Certificate Manager（ALBで終端）
  - パターンB: Let's Encrypt（Certbotで自動更新）

**推奨構成:**
- **AWS環境**: ALB + ACM（パターンA）
  - Route 53でドメイン管理
  - ACMで証明書管理（自動更新）
  - ALBでSSL/TLS終端
  - NginxはHTTPのみ
- **EC2単体**: Let's Encrypt（パターンB）
  - Certbotで証明書取得・更新
  - NginxでSSL/TLS終端

## Data Models

### 設定ファイル共有

```yaml
# Docker Volume: telegraf-config
# マウント先:
#   - Manager: /config (読み書き)
#   - Telegraf: /etc/telegraf (読み取り専用)

telegraf.conf:
  [[inputs.http_response]]
    urls = [
      "https://example.com",
      "https://example.org"
    ]
    response_timeout = "5s"
    method = "GET"
```

### Manager Database (SQLite)

```sql
-- users テーブル
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: 設定ファイル更新の原子性

*For any* URL更新操作、設定ファイルへの書き込みが完了するまで、Telegrafへのリロード指示は送信されない

**Validates: Requirements 11.1, 11.2**

### Property 2: SIGHUPシグナルの確実な送信

*For any* 設定ファイル更新が成功した場合、TelegrafコンテナにSIGHUPシグナルが送信される

**Validates: Requirements 11.2, 12.4**

### Property 3: Docker Socket アクセスの制限

*For any* Docker操作、managerコンテナは読み取り専用でDocker Socketにアクセスする

**Validates: Requirements 12.1**

### Property 4: 設定リロードのダウンタイムゼロ

*For any* Telegraf設定リロード、監視対象へのHTTPリクエストが途切れない

**Validates: Requirements 11.3**

### Property 5: エラー時の設定ロールバック

*For any* 設定ファイル更新が失敗した場合、以前の設定ファイルが保持される

**Validates: Requirements 11.5**

## Error Handling

### 1. 設定ファイル更新エラー

```rust
// 設定ファイル更新前にバックアップ
let backup_path = format!("{}.backup", config_path);
fs::copy(config_path, &backup_path)?;

match update_config(urls) {
    Ok(_) => {
        // 成功: バックアップ削除
        fs::remove_file(&backup_path)?;
    }
    Err(e) => {
        // 失敗: バックアップから復元
        fs::copy(&backup_path, config_path)?;
        return Err(e);
    }
}
```

### 2. Docker操作エラー

```rust
// Telegrafコンテナが存在しない場合
let output = Command::new("docker")
    .arg("kill")
    .arg("--signal=SIGHUP")
    .arg("telegraf")
    .output()?;

if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    if error.contains("No such container") {
        return Err(Error::string("Telegrafコンテナが見つかりません"));
    }
    return Err(Error::string(&format!("Docker操作エラー: {}", error)));
}
```

### 3. ボリューム共有エラー

```rust
// 共有ボリュームへのアクセス確認
let config_path = Path::new("/config/telegraf.conf");
if !config_path.exists() {
    return Err(Error::string(
        "設定ファイルが見つかりません。ボリュームマウントを確認してください"
    ));
}

// 書き込み権限の確認
if config_path.metadata()?.permissions().readonly() {
    return Err(Error::string(
        "設定ファイルへの書き込み権限がありません"
    ));
}
```

## Testing Strategy

### Unit Tests

**対象:**
- ConfigService: 設定ファイルの読み書き
- UrlValidationService: URL検証ロジック
- Telegrafコントローラー: Docker操作ロジック

**テストケース:**
- 正常系: 設定ファイルの読み書き
- 異常系: ファイルが存在しない、権限がない
- エッジケース: 空のURL配列、不正なURL形式

### Integration Tests

**対象:**
- Manager ↔ Telegraf: 設定更新とリロード
- Manager ↔ Docker Socket: コンテナ制御

**テストケース:**
- URL更新 → 設定ファイル更新 → SIGHUP送信 → Telegrafリロード
- Docker Socketが利用不可の場合のエラーハンドリング

### Property-Based Tests

**使用ライブラリ:** `proptest` (Rust)

**テスト対象:**
- Property 1: 設定ファイル更新の原子性
- Property 2: SIGHUPシグナルの確実な送信
- Property 5: エラー時の設定ロールバック

**設定:**
- 最小実行回数: 100回
- ランダムURL生成: 1-100個のURL

### End-to-End Tests

**シナリオ:**
1. システム全体を起動
2. 管理画面にログイン
3. URLを追加
4. Telegrafが新しいURLを監視開始
5. InfluxDBにデータが保存される
6. Grafanaでデータが表示される

## Deployment Strategy

### 移行手順

#### Phase 1: 新規リポジトリ作成

```bash
# 親ディレクトリに移動
cd ..

# 新規ディレクトリ作成
mkdir deadoralive-system
cd deadoralive-system

# Git初期化
git init
```

#### Phase 2: ファイル移行

```bash
# 管理系を移行
cp -r ../kiro_test/src manager/src
cp -r ../kiro_test/config manager/config
cp -r ../kiro_test/assets manager/assets
cp ../kiro_test/Cargo.toml manager/
cp ../kiro_test/Cargo.lock manager/

# 監視系を移行
cp -r ../deadoralive/telegraf monitoring/telegraf
cp -r ../deadoralive/influxdb monitoring/influxdb
cp -r ../deadoralive/grafana monitoring/grafana
```

#### Phase 3: 統合ファイル作成

```bash
# docker-compose.yml作成
# nginx/nginx.conf作成
# Makefile作成
# .env.example作成
# README.md作成
```

#### Phase 4: Dockerfile作成

```bash
# manager/Dockerfile作成（マルチステージビルド）
```

#### Phase 5: テスト

```bash
# ビルド
make build

# 起動
make up

# ヘルスチェック
make health

# ログ確認
make logs
```

#### Phase 6: コミット

```bash
git add .
git commit -m "Initial commit: Unified monitoring and management system"
git remote add origin <repository-url>
git push -u origin main
```

## Performance Considerations

### ビルド時間の最適化

- Dockerマルチステージビルドで依存関係をキャッシュ
- Cargo依存関係を先にビルド（レイヤーキャッシュ活用）

### 実行時パフォーマンス

- Nginxでgzip圧縮を有効化
- InfluxDBのデータ保持期間を適切に設定
- Telegrafの監視間隔を調整（デフォルト10秒）

### リソース使用量

- t4g.medium (2vCPU, 4GB RAM) で 300-500エンドポイント監視可能
- メモリ使用率: 60-70%
- CPU使用率: 50-70%

## Security Considerations

### Docker Socket アクセス

- managerコンテナのみがDocker Socketにアクセス
- 読み取り専用マウント（`:ro`）を推奨
- 最小権限の原則に従う

### 環境変数管理

- `.env`ファイルをgitignore
- 本番環境では強力なパスワードを使用
- JWT_SECRETは十分な長さのランダム文字列

### ネットワーク分離

- frontendネットワーク: Nginx, Manager, Grafana
- backendネットワーク: Manager, Telegraf, InfluxDB
- 不要な通信を制限

### SSL/TLS

- Let's Encryptで自動証明書取得
- HTTPSへの強制リダイレクト
- HSTSヘッダーの設定

## Maintenance and Operations

### バックアップ

```bash
# データベースバックアップ
docker-compose exec influxdb influx backup /backup

# 設定ファイルバックアップ
tar -czf backup-$(date +%Y%m%d).tar.gz monitoring/telegraf/telegraf.conf
```

### ログ管理

```bash
# ログローテーション設定
# Docker Composeのlogging設定で制限
```

### モニタリング

- CloudWatch Logsへのログ送信
- ヘルスチェックエンドポイントの監視
- アラート設定（InfluxDB + Grafana）

## Migration Checklist

- [ ] 新規ディレクトリ作成
- [ ] 管理系ファイル移行
- [ ] 監視系ファイル移行
- [ ] docker-compose.yml作成
- [ ] Nginx設定作成
- [ ] Makefile作成
- [ ] .env.example作成
- [ ] README.md作成
- [ ] Dockerfile作成
- [ ] .gitignore作成
- [ ] ドキュメント作成
- [ ] スクリプト作成
- [ ] ビルドテスト
- [ ] 起動テスト
- [ ] 統合テスト
- [ ] Git初期化
- [ ] 初回コミット


## AWS Certificate Manager + Route 53 対応

### 構成パターン

本システムは2つのデプロイパターンに対応します：

#### パターンA: AWS ALB + ACM（推奨）

**構成:**
```
Route 53 → ALB (SSL終端) → EC2:80 → Nginx → Backend
```

**Nginx設定（HTTP のみ）:**

```nginx
# nginx/nginx-aws.conf
events {
    worker_connections 1024;
}

http {
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    gzip on;
    gzip_types text/plain text/css application/json application/javascript;
    gzip_min_length 1000;

    limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=admin:10m rate=5r/s;

    # Grafana
    server {
        listen 80;
        server_name doa.com;

        location /health {
            access_log off;
            return 200 "healthy\n";
        }

        location / {
            proxy_pass http://grafana:3000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    # Manager
    server {
        listen 80;
        server_name admin.doa.com;

        location /health {
            access_log off;
            return 200 "healthy\n";
        }

        location / {
            proxy_pass http://manager:8081;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

**AWS設定:**
- Route 53: A/ALIASレコード → ALB
- ALB: HTTPS:443 → ACM証明書 → EC2:80
- Security Group: ALBからEC2:80を許可

#### パターンB: Let's Encrypt（EC2単体）

**構成:**
```
Route 53 → EC2:443 → Nginx (SSL終端) → Backend
```

**Nginx設定（HTTPS対応）:**

design.mdの既存のNginx設定を使用（SSL証明書あり）

### パフォーマンス最適化の詳細

#### 1. Gzip圧縮

**対象:** ネットワーク帯域幅とページ読み込み速度

**効果:**
- HTML/CSS/JavaScriptを60-80%圧縮
- データ転送量削減 → 通信コスト削減
- ページ読み込み時間短縮

**理由:**
- Grafanaは大量のJavaScriptを含む（500KB → 100KB）
- 特にモバイルや低速回線で効果大
- CPU負荷はわずか

#### 2. HTTP/2

**対象:** 複数リソースの並列読み込み

**効果:**
- 1つのTCP接続で複数ファイルを同時転送
- ヘッダー圧縮でオーバーヘッド削減
- 読み込み時間を最大10倍高速化

**理由:**
- Grafanaは多数のリソース（20+ファイル）を読み込む
- HTTP/1.1の6-8接続制限を回避
- 最新ブラウザは全てHTTP/2対応

#### 3. プロキシバッファリング

**対象:** バックエンドサーバーの負荷

**効果:**
- Nginxがレスポンスをバッファリング
- バックエンドは即座に次のリクエストを処理
- 低速クライアントの影響を遮断

**理由:**
- バックエンドの同時接続数を削減
- レスポンス時間の安定化
- スループット向上

**具体例:**
```
バッファリングなし: Manager → 遅いクライアント (10秒接続維持)
バッファリングあり: Manager → Nginx (0.1秒) → Nginx → クライアント (10秒)
```

### 推奨構成の選択

| 項目 | AWS ALB + ACM | Let's Encrypt |
|------|--------------|---------------|
| **コスト** | +$16/月（ALB） | $0 |
| **証明書管理** | 自動（ACM） | 手動更新必要 |
| **スケーラビリティ** | 高（ALB） | 低（単一EC2） |
| **セットアップ** | 複雑 | シンプル |
| **推奨用途** | 本番環境 | 開発/小規模 |

**推奨:** 本番環境ではAWS ALB + ACMを使用
