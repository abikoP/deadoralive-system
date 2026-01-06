#!/bin/bash
set -e

# Dead or Alive System - バックアップスクリプト
# データベースと設定ファイルをバックアップ

echo "========================================="
echo "Dead or Alive System - バックアップ"
echo "========================================="
echo ""

# バックアップディレクトリ作成
BACKUP_DIR="backups/backup-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "💾 バックアップ先: $BACKUP_DIR"
echo ""

# InfluxDBバックアップ
echo "📊 InfluxDBをバックアップしています..."
if docker-compose ps | grep -q influxdb; then
    docker-compose exec -T influxdb influx backup /backup
    docker cp $(docker-compose ps -q influxdb):/backup "$BACKUP_DIR/influxdb"
    echo "✅ InfluxDB: 完了"
else
    echo "⚠️  InfluxDBコンテナが起動していません"
fi
echo ""

# Managerデータベースバックアップ
echo "🗄️  Managerデータベースをバックアップしています..."
if docker-compose ps | grep -q manager; then
    docker-compose exec -T manager cp /data/db.sqlite /data/db.sqlite.backup
    docker cp $(docker-compose ps -q manager):/data/db.sqlite.backup "$BACKUP_DIR/manager.sqlite"
    echo "✅ Manager: 完了"
else
    echo "⚠️  Managerコンテナが起動していません"
fi
echo ""

# Grafanaダッシュボードバックアップ
echo "📈 Grafanaダッシュボードをバックアップしています..."
if docker-compose ps | grep -q grafana; then
    docker cp $(docker-compose ps -q grafana):/var/lib/grafana "$BACKUP_DIR/grafana" 2>/dev/null || true
    echo "✅ Grafana: 完了"
else
    echo "⚠️  Grafanaコンテナが起動していません"
fi
echo ""

# Telegraf設定バックアップ
echo "⚙️  Telegraf設定をバックアップしています..."
if [ -f monitoring/telegraf/telegraf.conf ]; then
    cp monitoring/telegraf/telegraf.conf "$BACKUP_DIR/telegraf.conf"
    echo "✅ Telegraf設定: 完了"
fi
echo ""

# 環境変数バックアップ（パスワードは除外）
echo "🔐 環境変数をバックアップしています..."
if [ -f .env ]; then
    # パスワード系の環境変数を除外してバックアップ
    grep -v -E "(PASSWORD|SECRET|TOKEN)" .env > "$BACKUP_DIR/.env.backup" || true
    echo "✅ 環境変数: 完了（パスワードは除外）"
fi
echo ""

# バックアップサイズ確認
BACKUP_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
echo "========================================="
echo "✅ バックアップ完了"
echo "========================================="
echo ""
echo "バックアップ先: $BACKUP_DIR"
echo "サイズ: $BACKUP_SIZE"
echo ""
echo "💡 復元するには以下を実行:"
echo "   ./scripts/restore.sh $BACKUP_DIR"
