@echo off
rem Preview the Legendary desktop shell running the PRODUCTION renderer build
rem (frontend\dist), no hot reload - closest to a shipped app.
rem
rem   scripts\preview-desktop.bat
rem   scripts\preview-desktop.bat --release   (extra args pass to cargo run)
rem
rem - Builds frontend\dist (npm run build).
rem - Serves the dist at http://localhost:4173 so the same build can also be
rem   inspected in a plain browser tab.
rem - Runs `cargo run -p legendary-desktop --features custom-protocol`, which
rem   makes Tauri load the dist itself (bundled mode) instead of the dev
rem   server on :5173.
rem
rem For hot-reload development see dev-desktop.bat.

setlocal enabledelayedexpansion
set "ROOT=%~dp0.."

rem ---- 1. Frontend deps + production build -----------------------------------
if not exist "%ROOT%\frontend\node_modules" (
  echo [preview-desktop] installing frontend dependencies...
  call npm install --prefix "%ROOT%\frontend"
  if errorlevel 1 (
    echo [preview-desktop] npm install failed
    exit /b 1
  )
)

echo [preview-desktop] building frontend\dist...
call npm run build --prefix "%ROOT%\frontend"
if errorlevel 1 (
  echo [preview-desktop] frontend build failed
  exit /b 1
)

rem ---- 2. Static browser preview of dist (optional convenience) ---------------
set STARTED_PREVIEW=0
netstat -ano 2>nul | findstr ":4173" | findstr "LISTENING" >nul 2>&1
if errorlevel 1 (
  echo [preview-desktop] serving dist at http://localhost:4173 for browser preview...
  start "legendary-preview" /min cmd /c "npm run preview --prefix %ROOT%\frontend"
  set STARTED_PREVIEW=1
  timeout /t 2 /nobreak >nul
)

rem ---- 3. Desktop shell in bundled mode ----------------------------------------
echo [preview-desktop] launching Legendary desktop shell (production renderer)...
cargo run -p legendary-desktop --features custom-protocol %*
set CARGO_RC=%errorlevel%

if "%STARTED_PREVIEW%"=="1" (
  echo [preview-desktop] stopping static preview server
  taskkill /F /T /FI "WINDOWTITLE eq legendary-preview*" >nul 2>&1
)

endlocal & exit /b %CARGO_RC%
