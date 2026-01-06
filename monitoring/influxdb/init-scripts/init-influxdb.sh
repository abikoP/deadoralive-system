#!/bin/bash
set -e

# Wait for InfluxDB to be ready
until influx ping; do
  echo "Waiting for InfluxDB..."
  sleep 2
done

# Create database if it doesn't exist
influx -execute "CREATE DATABASE IF NOT EXISTS monitoring"

# Set retention policy
influx -execute "CREATE RETENTION POLICY \"${INFLUXDB_RETENTION:-90d}\" ON monitoring DURATION ${INFLUXDB_RETENTION:-90d} REPLICATION 1 DEFAULT"

echo "InfluxDB initialization complete"
