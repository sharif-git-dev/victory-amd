@echo off
setlocal
title victory-amd (AMD RDNA 4 / RX 9000 Stable Preset)
cd /d "%~dp0.."

if "%~1"=="" (
    echo Usage: scripts\launch-amd-stable.cmd "D:\path\to\title.exe"
    echo Or edit your own local batch wrapper pointing at your executable.
    exit /b 1
)

set VICTORY_HOST_MAX_WAVE=32
set VICTORY_RELAY=
set VICTORY_LATE_WAIT=
set VICTORY_D3D12_DRED=1
set VICTORY_LOG=info

if exist ".\target\diag\victory.exe" (
    ".\target\diag\victory.exe" run "%~1" --execute
) else (
    ".\target\release\victory.exe" run "%~1" --execute
)
