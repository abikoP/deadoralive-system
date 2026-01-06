#!/bin/bash
set -e

# Dead or Alive System - リストアスクリプト
# バックアップからデータを復元

echo "========================================="
echo "Dead or Alive System - リストア"
echo "========================================="
echo ""

# 引数チェック
if [ $# -eq 0 ]; then
    echo "❌ エラー: バックアップディレクトリを指定してください"
    echo ""
    echo "使用方法:"
    echo "  ./scripts/restore.sh <backup-directory>"
    echo ""
    echo "利用可能なバックアップ:"
    ls -1d backups/backup-* 2>/dev/null || echo "  （バックアップが見つかりません）"
    exit 1
fi

BACKUP_DIR="$1"

# バックアップディレクトリ存在チェック
if [ ! -d "$BACKUP_DIR" ]; then
    echo "❌ エラー: バックアップディレクトリが見つかりません: $BACKUP_DIR"
    exit 1
fi

echo "📥 復元元: $BACKUP_DIR"
echo ""

# 確認プロンプト
echo "⚠️  警告: 現在のデータが上書きされます"
read -p "続行しますか? [y/N]: " confirm
if [ "$confirm" != "y" ]; then
    echo "❌ キャンセルしました"
    exit 0
fi
echo ""

# サービス停止
echo "🛑 サービスを停止しています..."
docker-compose down
echo "✅ 停止完了"
echo ""

# InfluxDBリストア
if [ -d "$BACKUP_DIR/influxdb" ]; then
    echo "📊 InfluxDBを復元しています..."
    docker-compose up -d influxdb
    sleep 10  # InfluxDB起動を待つ
    docker cp "$BACKUP_DIR/influxdb" $(docker-compose ps -q influxdb):/backup
    docker-compose exec -T influxdb influx restore /backup
    echo "✅ InfluxDB: 完了"
    docker-compose down
else
    echo "⚠️  InfluxDBバックアップが見つかりません"
fi
echo ""

# Managerデータベースリストア
if [ -f "$BACKUP_DIR/manager.sqlite" ]; then
    echo "🗄️  Managerデータベースを復元しています..."
    docker-compose up -d manager
    sleep 5  # Manager起動を待つ
    docker cp "$BACKUP_DIR/manager.sqlite" $(docker-compose ps -q manager):/data/db.sqlite
    echo "✅ Manager: 完了"
    docker-compose down
else
    echo "⚠️  Managerバックアップが見つかりません"
fi
echo ""

# Grafanaダッシュボードリストア
if [ -d "$BACKUP_DIR/grafana" ]; then
    echo "📈 Grafanaダッシュボードを復元しています..."
    docker-compose up -d grafana
    sleep 5  # Grafana起動を待つ
    docker cp "$BACKUP_DIR/grafana" $(docker-compose ps -q grafana):/var/lib/
    echo "✅ Grafana: 完了"
    docker-compose down
else
    echo "⚠️  Grafanaバックアップが見つかりません"
fi
echo ""

# Telegraf設定リストア
if [ -f "$BACKUP_DIR/telegraf.conf" ]; then
    echo "⚙️  Telegraf設定を復元しています..."
    cp "$BACKUP_DIR/telegraf.conf" monitoring/telegraf/telegraf.conf
    echo "✅ Telegraf設定: 完了"
fi
echo ""

# サービス再起動
echo "🚀 サービスを起動しています..."
docker-compose up -d
echo "✅ 起動完了"
echo ""

# ヘルスチェック
echo "🏥 ヘルスチェックを実行しています..."
sleep 10  # サービス起動を待つ

./scripts/health-check.sh

echo ""
echo "========================================="
echo "✅ リストア完了"
echo "========================================="
