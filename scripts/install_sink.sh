#!/bin/bash

WEBHOOK_VERSION="v0.7.0"
POSTGRES_VERSION="v0.8.0"
CONSOLE_VERSION="v0.6.1"

case $PLATFORM in
    "macos-aarch64")
        WEBHOOK_FILE="sink-webhook-aarch64-macos"
        POSTGRES_FILE="sink-postgres-aarch64-macos"
        CONSOLE_FILE="sink-console-aarch64-macos"
        ;;
    "linux-x86_64")
        WEBHOOK_FILE="sink-webhook-x86_64-linux"
        POSTGRES_FILE="sink-postgres-x86_64-linux"
        CONSOLE_FILE="sink-console-x86_64-linux"
        ;;
    "linux-aarch64")
        WEBHOOK_FILE="sink-webhook-aarch64-linux"
        POSTGRES_FILE="sink-postgres-aarch64-linux"
        CONSOLE_FILE="sink-console-aarch64-linux"
        ;;
    *)
        echo "Unsupported platform: $PLATFORM"
        echo "Supported platforms: linux-x86_64, linux-aarch64, macos-aarch64"
        exit 1
        ;;
esac

# Download and install sink-webhook
echo "Installing sink-webhook..."
wget "https://github.com/apibara/dna/releases/download/sink-webhook/${WEBHOOK_VERSION}/${WEBHOOK_FILE}.gz"
gunzip "${WEBHOOK_FILE}.gz"
sudo cp "${WEBHOOK_FILE}" ./bin/sink-webhook
sudo chmod 777 ./bin/sink-webhook
rm "${WEBHOOK_FILE}"

# Download and install sink-postgres
echo "Installing sink-postgres..."
wget "https://github.com/apibara/dna/releases/download/sink-postgres/${POSTGRES_VERSION}/${POSTGRES_FILE}.gz"
gunzip "${POSTGRES_FILE}.gz"
sudo cp "${POSTGRES_FILE}" ./bin/sink-postgres
sudo chmod 777 ./bin/sink-postgres
rm "${POSTGRES_FILE}"

# Download and install sink-console
echo "Installing sink-console..."
wget "https://github.com/apibara/dna/releases/download/sink-console/${CONSOLE_VERSION}/${CONSOLE_FILE}.gz"
gunzip "${CONSOLE_FILE}.gz"
sudo cp "${CONSOLE_FILE}" ./bin/sink-console
sudo chmod 777 ./bin/sink-console
rm "${CONSOLE_FILE}"

echo "Installation completed successfully!"