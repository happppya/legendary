@echo off
rem Launch the Legendary desktop shell against a live Vite dev server.
rem
rem   scripts\dev-desktop.bat
rem   scripts\dev-desktop.bat --release     (extra args pass through to cargo run)
rem
rem - Installs frontend deps if node_modules is missing.
rem - Starts the Vite dev server on :5173 unless one is already listening
rem   (it is REUSED and not stopped afterwards).
rem - Runs `cargo run -p legendary-desktop`, which loads the renderer from
rem   http://localhost:5173 (Tauri dev mode) over IPC to legend_core.
rem
rem For the production-renderer preview see preview-desktop.bat.

setlocal enabledelayedexpansion
set "ROOT=%~dp0.."
set PORT=5173

rem ---- 1. Frontend deps ----------------------------------------------------
if not exist "%ROOT%\frontend\node_modules" (
  echo [dev-desktop] installing frontend dependencies...
  call npm install --prefix "%ROOT%\frontend"
  if errorlevel 1 (
    echo [dev-desktop] npm install failed
    exit /b 1
  )
)

rem ---- 2. Vite dev server (reuse one already up) ----------------------------
set PORT_UP=0
netstat -ano 2>nul | findstr ":%PORT%" | findstr "LISTENING" >nul 2>&1
if not errorlevel 1 set PORT_UP=1

if "%PORT_UP%"=="1" (
  echo [dev-desktop] reusing Vite dev server already listening on :%PORT%
  goto :run
)

echo [dev-desktop] starting Vite dev server on :%PORT%...
start "legendary-vite" /min cmd /c "npm run dev --prefix %ROOT%\frontend"

set /a TRIES=0
:waitloop
netstat -ano 2>nul | findstr ":%PORT%" | findstr "LISTENING" >nul 2>&1
if not errorlevel 1 goto :portup
set /a TRIES+=1
if %TRIES% GEQ 30 (
  echo [dev-desktop] warning: Vite not detected on :%PORT% yet, launching anyway
  goto :run
)
timeout /t 1 /nobreak >nul
goto :waitloop

:portup
echo [dev-desktop] Vite is up on :%PORT%

rem ---- 3. Desktop shell ------------------------------------------------------
:run
echo [dev-desktop] launching Legendary desktop shell (dev renderer)...
cargo run -p legendary-desktop %*
set CARGO_RC=%errorlevel%

rem Stop the Vite window only if WE started it.
if "%PORT_UP%"=="0" (
  echo [dev-desktop] stopping Vite dev server
  taskkill /F /T /FI "WINDOWTITLE eq legendary-vite*" >nul 2>&1
)

endlocal & exit /b %CARGO_RC%
