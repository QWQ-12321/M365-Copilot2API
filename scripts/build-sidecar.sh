#!/usr/bin/env bash
# 编译 Go sidecar 二进制文件到 Tauri 的 binaries/ 目录
# 用法: ./scripts/build-sidecar.sh [TARGET]
#   TARGET: windows-x86_64 (默认), linux-x86_64, darwin-x86_64, darwin-arm64

set -euo pipefail

TARGET="${1:-windows-x86_64}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARIES_DIR="$PROJECT_ROOT/src-tauri/binaries"

mkdir -p "$BINARIES_DIR"

echo "==> 编译 Go sidecar (target: $TARGET)"
echo "    入口: cmd/server/main.go"

case "$TARGET" in
  windows-x86_64)
    export GOOS=windows
    export GOARCH=amd64
    export CGO_ENABLED=0
    OUT="$BINARIES_DIR/m365-copilot2api-x86_64-pc-windows-msvc.exe"
    ;;
  linux-x86_64)
    export GOOS=linux
    export GOARCH=amd64
    export CGO_ENABLED=0
    OUT="$BINARIES_DIR/m365-copilot2api-x86_64-unknown-linux-gnu"
    ;;
  darwin-x86_64)
    export GOOS=darwin
    export GOARCH=amd64
    export CGO_ENABLED=0
    OUT="$BINARIES_DIR/m365-copilot2api-x86_64-apple-darwin"
    ;;
  darwin-arm64)
    export GOOS=darwin
    export GOARCH=arm64
    export CGO_ENABLED=0
    OUT="$BINARIES_DIR/m365-copilot2api-aarch64-apple-darwin"
    ;;
  *)
    echo "未知 target: $TARGET"
    echo "可用: windows-x86_64, linux-x86_64, darwin-x86_64, darwin-arm64"
    exit 1
    ;;
esac

echo "    GOOS=$GOOS GOARCH=$GOARCH CGO_ENABLED=$CGO_ENABLED"
echo "    输出: $OUT"

cd "$PROJECT_ROOT"
go build -ldflags="-s -w" -o "$OUT" ./cmd/server

SIZE=$(du -h "$OUT" | cut -f1)
echo "==> 完成: $OUT ($SIZE)"
