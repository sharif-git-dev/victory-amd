@echo off
setlocal
title victory-amd experimental diagnostics
if "%~1"=="" (
    echo Usage: launch-amd.cmd "D:\path\to\title.exe"
    exit /b 1
)
set "VICTORY_TITLE=%~f1"
set "VICTORY_HOST_MAX_WAVE=32"
set "VICTORY_RELAY="
set "VICTORY_LATE_WAIT="
set "VICTORY_D3D12_DRED=1"
set "VICTORY_LOG=info,victory_gpu::pipeline=warn,victory_gpu::copystat=warn"
set VICTORY_SKIP_PIPELINES=f16876929fa64f9f,a1266cc7391ae0d6,a73284b60aa303ed,203f78efece5bbdf,4e69dc8770bf27fc,1c02f8131c508c75,501f0865fdf45dfd,61d7c3333b73892d,1e4947eada3d9ee3,27ce027e7188bed3,52ab310c43b9f99c,f018c205b5199d39,077869cd9b1456ee,a4df980a1c65fdbe,072f5fe96efd27e1,d0130073bb6f7a7d,a2d880ad5dea1f45,3a6594961b9247e7,a62731e90739f195,b7a7fb5ba5934a89,a0ce4bef0ae97c3f,7fb216a0c6dee3a9,dbba7e080e26ec20,8a13d3da7f64b8d1,efdb8133aabc37fe,2e28ea6fb9dad3c2,d2c52fcd936985b4,3fa1fda35402c93f,231d2daa66789312,0ae69044bfd15035,1f1fbd86759b3da2
if exist "%~dp0victory.exe" (
    set "VICTORY_RUNTIME=%~dp0victory.exe"
    cd /d "%~dp0"
) else if exist "%~dp0..\target\diag\victory.exe" (
    set "VICTORY_RUNTIME=%~dp0..\target\diag\victory.exe"
    cd /d "%~dp0.."
) else (
    echo Missing diagnostics runtime. Build: cargo build --profile diag -p victory --features diagnostics
    exit /b 1
)
"%VICTORY_RUNTIME%" run "%VICTORY_TITLE%" --execute
exit /b %errorlevel%
