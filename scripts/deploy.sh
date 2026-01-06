#!/bin/bash
set -e

# Dead or Alive System - デプロイスクリプト
# 本番環境へのデプロイを自動化

echo "========================================="
echo "Dead or Alive System - デプロイ"
echo "========================================="
echo ""

# 環境変数チェック
if [ ! -f .env ]; then
    echo "❌ エラー: .envファイルが見つかりません"
    echo "💡 .env.exampleをコピーして.envを作成してください"
    exit 1
fi

# Gitリポジトリチェック
if [ -d .git ]; then
    echo "📦 Gitリポジトリを更新しています..."
    git pull origin main
    echo "✅ 更新完了"
    echo ""
fi

# Dockerイメージのビルド
echo "🔨 Dockerイメージをビルドしています..."
docker-compose build --no-cache
echo "✅ ビルド完了"
echo ""

# 既存のコンテナを停止
echo "🛑 既存のコンテナを停止しています..."
docker-compose down
echo "✅ 停止完了"
echo ""

# データベースのバックアップ
echo "💾 データベースをバックアップしています..."
if [ -d backups ]; then
    BACKUP_DIR="backups/pre-deploy-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$BACKUP_DIR"
    
    # InfluxDBバックアップ（コンテナが起動している場合）
    if docker-compose ps | grep -q influxdb; then
        docker-compose exec -T influxdb influx backup /backup 2>/dev/null || true
        docker cp $(docker-compose ps -q influxdb):/backup "$BACKUP_DIR/influxdb" 2>/dev/null || true
    fi
    
    # Managerバックアップ（コンテナが起動している場合）
    if docker-compose ps | grep -q manager; then
        docker cp $(docker-compose ps -q manager):/data/db.sqlite "$BACKUP_DIR/manager.sqlite" 2>/dev/null || true
    fi
    
    echo "✅ バックアップ完了: $BACKUP_DIR"
else
    echo "⚠️  backupsディレクトリが存在しないため、バックアップをスキップします"
fi
echo ""

# 新しいコンテナを起動
echo "🚀 新しいコンテナを起動しています..."
docker-compose up -d
echo "✅ 起動完了"
echo ""

# ヘルスチェック
echo "🏥 ヘルスチェックを実行しています..."
sleep 10  # サービス起動を待つ

HEALTH_OK=true

# Nginx
if curl -f http://localhost/health >/dev/null 2>&1; then
    echo "  ✅ Nginx: OK"
else
    echo "  ❌ Nginx: NG"
    HEALTH_OK=false
fi

# Manager
if curl -f http://localhost:8081/health >/dev/null 2>&1; then
    echo "  ✅ Manager: OK"
else
    echo "  ❌ Manager: NG"
    HEALTH_OK=false
fi

# InfluxDB
if docker-compose exec -T influxdb influx ping >/dev/null 2>&1; then
    echo "  ✅ InfluxDB: OK"
else
    echo "  ❌ InfluxDB: NG"
    HEALTH_OK=false
fi

# Grafana
if curl -f http://localhost:3000/api/health >/dev/null 2>&1; then
    echo "  ✅ Grafana: OK"
else
    echo "  ❌ Grafana: NG"
    HEALTH_OK=false
fi

echo ""

if [ "$HEALTH_OK" = true ]; then
    echo "========================================="
    echo "✅ デプロイ成功"
    echo "========================================="
    echo ""
    echo "アクセス先:"
    echo "  - Grafana:     https://doa.com"
    echo "  - 管理画面:    https://admin.doa.com"
    exit 0
else
    echo "========================================="
    echo "⚠️  デプロイ完了（一部サービスに問題あり）"
    echo "========================================="
    echo ""
    echo "ログを確認してください:"
    echo "  docker-compose logs"
    exit 1
fi
