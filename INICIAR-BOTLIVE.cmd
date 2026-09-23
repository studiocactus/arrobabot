@echo off
setlocal
cd /d "%~dp0"
if exist "entrega\BotLive.exe" (
  start "" "entrega\BotLive.exe"
  exit /b 0
)
if exist "src-tauri\target\release\botlive.exe" (
  start "" "src-tauri\target\release\botlive.exe"
  exit /b 0
)
echo O executavel ainda nao foi gerado. Consulte o README.md.
pause
