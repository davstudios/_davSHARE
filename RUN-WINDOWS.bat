@echo off
setlocal
cd /d "%~dp0"
title _davSHARE v26.10.2

echo ========================================
echo        _davSHARE v26.10.2
echo ========================================
echo.
where node >nul 2>nul || goto node_error
where npm >nul 2>nul || goto npm_error
where cargo >nul 2>nul || goto cargo_error
echo Controllo sessioni di sviluppo precedenti...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\prepare-windows-dev.ps1" -Port 17462
if errorlevel 1 goto port_error
echo Sincronizzazione dipendenze npm...
call npm install --no-audit --no-fund
if errorlevel 1 goto install_error
echo Avvio _davSHARE...
call npm run desktop
if errorlevel 1 goto app_error
exit /b 0

:node_error
echo ERRORE: Node.js non trovato.
goto fail
:npm_error
echo ERRORE: npm non trovato.
goto fail
:cargo_error
echo ERRORE: Rust/Cargo non trovato.
goto fail
:install_error
echo ERRORE: installazione npm non completata.
goto fail
:port_error
echo ERRORE: impossibile liberare la sessione di sviluppo precedente.
goto fail
:app_error
echo ERRORE: _davSHARE non e riuscito ad avviarsi.
goto fail
:fail
echo.
echo Copia qui in chat il messaggio mostrato sopra.
pause
exit /b 1
