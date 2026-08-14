@echo off
REM 编译 Go sidecar 二进制文件到 Tauri 的 binaries/ 目录 (Windows 批处理版)
REM 用法: scripts\build-sidecar.bat [TARGET]
REM   TARGET: windows-x86_64 (默认), linux-x86_64

setlocal enabledelayedexpansion

set "TARGET=%~1"
if "%TARGET%"=="" set "TARGET=windows-x86_64"

set "PROJECT_ROOT=%~dp0.."
set "BINARIES_DIR=%PROJECT_ROOT%\src-tauri\binaries"

if not exist "%BINARIES_DIR%" mkdir "%BINARIES_DIR%"

echo ==> 编译 Go sidecar (target: %TARGET%)

if "%TARGET%"=="windows-x86_64" (
    set "GOOS=windows"
    set "GOARCH=amd64"
    set "CGO_ENABLED=0"
    set "OUT=%BINARIES_DIR%\m365-copilot2api-x86_64-pc-windows-msvc.exe"
) else if "%TARGET%"=="linux-x86_64" (
    set "GOOS=linux"
    set "GOARCH=amd64"
    set "CGO_ENABLED=0"
    set "OUT=%BINARIES_DIR%\m365-copilot2api-x86_64-unknown-linux-gnu"
) else (
    echo 未知 target: %TARGET%
    echo 可用: windows-x86_64, linux-x86_64
    exit /b 1
)

echo     GOOS=%GOOS% GOARCH=%GOARCH% CGO_ENABLED=%CGO_ENABLED%
echo     输出: %OUT%

cd /d "%PROJECT_ROOT%"
go build -ldflags="-s -w" -o "%OUT%" ./cmd/server

if %errorlevel% neq 0 (
    echo [错误] Go 编译失败
    exit /b 1
)

echo ==> 完成: %OUT%
