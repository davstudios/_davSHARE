@echo off
setlocal EnableExtensions
chcp 65001 >nul
title _davCLEAN-PACKAGE v2

echo ========================================
echo       _davCLEAN-PACKAGE v2
echo ========================================
echo.
echo Pulisce un progetto _davstudios Tauri/Vite
echo e crea uno ZIP sorgente leggero.
echo.

set "SCRIPT_DIR=%~dp0"
set "TARGET=%~1"

if not defined TARGET (
    if exist "%SCRIPT_DIR%package.json" if exist "%SCRIPT_DIR%src-tauri\" (
        set "TARGET=%SCRIPT_DIR%"
    )
)

if not defined TARGET (
    echo Trascina la cartella del progetto sopra questo file .bat
    echo oppure metti il .bat direttamente dentro il progetto.
    echo.
    pause
    exit /b 1
)

for %%A in ("%TARGET%") do set "PROJECT=%%~fA"
if "%PROJECT:~-1%"=="\" set "PROJECT=%PROJECT:~0,-1%"

for %%A in ("%PROJECT%") do (
    set "PROJECT_NAME=%%~nxA"
    set "PARENT_DIR=%%~dpA"
)

if not exist "%PROJECT%\package.json" (
    echo [ERRORE] package.json non trovato:
    echo %PROJECT%
    echo.
    pause
    exit /b 1
)

if not exist "%PROJECT%\src-tauri\" (
    echo [ERRORE] src-tauri non trovato:
    echo %PROJECT%
    echo.
    pause
    exit /b 1
)

echo Progetto:
echo %PROJECT%
echo.
echo Chiusura processi di sviluppo collegati alla cartella...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$p=[regex]::Escape('%PROJECT%'); Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -and $_.CommandLine -match $p -and ($_.Name -match 'node|cargo|rustc|dav.*\.exe|tauri') } | ForEach-Object { try { taskkill /PID $_.ProcessId /T /F | Out-Null } catch {} }" >nul 2>&1

timeout /t 1 /nobreak >nul

echo.
echo Pulizia cartelle rigenerabili...

call :REMOVE_DIR "%PROJECT%\node_modules"
call :REMOVE_DIR "%PROJECT%\src-tauri\target"
call :REMOVE_DIR "%PROJECT%\target"
call :REMOVE_DIR "%PROJECT%\dist"
call :REMOVE_DIR "%PROJECT%\.vite"
call :REMOVE_DIR "%PROJECT%\.cache"
call :REMOVE_DIR "%PROJECT%\coverage"

echo.
echo Preparazione pacchetto...

set "OUT_DIR=%PARENT_DIR%_davPACKAGES"
if not exist "%OUT_DIR%" mkdir "%OUT_DIR%"

set "ZIP_PATH=%OUT_DIR%\%PROJECT_NAME%-SOURCE.zip"

if exist "%ZIP_PATH%" del /f /q "%ZIP_PATH%"

set "STAGE=%TEMP%\davpackage_%RANDOM%_%RANDOM%"
set "STAGE_PROJECT=%STAGE%\%PROJECT_NAME%"

mkdir "%STAGE_PROJECT%" >nul 2>&1

echo Copia sorgenti...
robocopy "%PROJECT%" "%STAGE_PROJECT%" /E /R:1 /W:1 /NFL /NDL /NJH /NJS /NP ^
  /XD ".git" "node_modules" "target" "dist" ".vite" ".cache" "coverage" "_davPACKAGES" ^
  /XF "npm-debug.log" "yarn-error.log" "pnpm-debug.log" >nul

set "ROBO=%ERRORLEVEL%"
if %ROBO% GEQ 8 (
    echo [ERRORE] Copia dei sorgenti fallita. Codice Robocopy: %ROBO%
    rmdir /s /q "%STAGE%" >nul 2>&1
    pause
    exit /b 1
)

echo Compressione ZIP...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ErrorActionPreference='Stop'; Compress-Archive -LiteralPath '%STAGE_PROJECT%' -DestinationPath '%ZIP_PATH%' -CompressionLevel Optimal -Force"

if errorlevel 1 (
    echo.
    echo [ERRORE] Creazione ZIP fallita.
    if exist "%STAGE%" rmdir /s /q "%STAGE%" >nul 2>&1
    pause
    exit /b 1
)

if exist "%STAGE%" rmdir /s /q "%STAGE%" >nul 2>&1

if not exist "%ZIP_PATH%" (
    echo.
    echo [ERRORE] Lo ZIP non risulta creato.
    pause
    exit /b 1
)

for %%Z in ("%ZIP_PATH%") do set "ZIP_SIZE=%%~zZ"

echo.
echo ========================================
echo FATTO
echo ========================================
echo.
echo ZIP creato:
echo %ZIP_PATH%
echo.
echo Dimensione: %ZIP_SIZE% byte
echo.
echo Apro la cartella di destinazione...
start "" explorer.exe /select,"%ZIP_PATH%"
echo.
pause
exit /b 0


:REMOVE_DIR
if exist "%~1\" (
    echo - %~1
    rmdir /s /q "%~1" >nul 2>&1
    if exist "%~1\" (
        echo   [ATTENZIONE] Non eliminata completamente.
    ) else (
        echo   [OK]
    )
)
exit /b
