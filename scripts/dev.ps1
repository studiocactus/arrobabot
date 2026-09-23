param([switch]$Build,[switch]$Test)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $projectRoot
if (Test-Path -LiteralPath "$projectRoot\.tools\cargo\bin\cargo.exe") {
  $env:RUSTUP_HOME = Join-Path $projectRoot '.tools\rustup'
  $env:CARGO_HOME = Join-Path $projectRoot '.tools\cargo'
  $env:PATH = (Join-Path $projectRoot '.tools\cargo\bin') + ';' + $env:PATH
}
$env:CARGO_BUILD_JOBS = '1'
if ($Test) {
  & npm.cmd test
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  & cargo test --manifest-path src-tauri/Cargo.toml -j 1
} elseif ($Build) {
  & npm.cmd run package
} else {
  & npm.cmd run desktop
}
exit $LASTEXITCODE
