#!/bin/bash
set -e

# Wait for fake-gcs to be ready
until curl -s http://localhost:4443/storage/v1/b > /dev/null; do
  echo 'Waiting for fake-gcs...'
  sleep 1
done

# Create the test bucket
curl -X POST \
  --data-binary '{"name":"test-bucket"}' \
  -H "Content-Type: application/json" \
  "http://localhost:4443/storage/v1/b"

echo "GCS emulator initialized with test-bucket" 