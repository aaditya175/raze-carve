@echo off
setlocal

set "WEBSITE_DIR=%~dp0"
set "URL_FILE=%WEBSITE_DIR%tunnel_url.txt"
set "LOG_FILE=%TEMP%\cf_tunnel_err.log"

if exist "%URL_FILE%" del /f /q "%URL_FILE%"
if exist "%LOG_FILE%" del /f /q "%LOG_FILE%"

taskkill /f /im cloudflared.exe >nul 2>&1

echo Starting Vite in website...
start /b "" npx --prefix "%WEBSITE_DIR%." vite "%WEBSITE_DIR%." --host 127.0.0.1 --port 5174 --strictPort

timeout /t 3 /nobreak >nul

echo Starting Cloudflare Tunnel...
start /b "" "%TEMP%\cloudflared.exe" tunnel --url http://127.0.0.1:5174 2>"%LOG_FILE%"

for /l %%i in (1,1,30) do (
    timeout /t 1 /nobreak >nul
    if exist "%LOG_FILE%" (
        findstr /r "https://.*\.trycloudflare\.com" "%LOG_FILE%" > "%URL_FILE%" 2>nul
        for %%A in ("%URL_FILE%") do if %%~zA gtr 0 goto :found
    )
)

:found
echo Tunnel started successfully.
