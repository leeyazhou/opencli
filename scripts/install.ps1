$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent $PSScriptRoot
$TargetDir = Join-Path $HOME '.local\bin'

New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null

cargo build --release --manifest-path (Join-Path $RootDir 'Cargo.toml')
Copy-Item (Join-Path $RootDir 'target\release\opencli.exe') (Join-Path $TargetDir 'opencli.exe') -Force

Write-Output "Installed opencli to $(Join-Path $TargetDir 'opencli.exe')"
