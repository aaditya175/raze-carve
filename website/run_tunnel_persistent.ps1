$ErrorActionPreference = "Continue"
$workDir = "c:\Users\AADITYA CHAUHAN\Desktop\New folder (3)\SecureForge-main"
$logFile = "$env:TEMP\cf_website.log"
$outLog = "$env:TEMP\cf_website_out.log"
$urlFile = "$workDir\website\tunnel_url.txt"

if (Test-Path $logFile) { Remove-Item $logFile -Force }
if (Test-Path $outLog) { Remove-Item $outLog -Force }
if (Test-Path $urlFile) { Remove-Item $urlFile -Force }

# Kill existing cloudflared
Get-Process -Name cloudflared -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Kill any existing process on 5174
$portConns = Get-NetTCPConnection -LocalPort 5174 -ErrorAction SilentlyContinue
foreach ($conn in $portConns) {
    if ($conn.OwningProcess -gt 0) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
    }
}
Start-Sleep -Seconds 1

# Start Vite
Start-Process -FilePath "npx.cmd" -ArgumentList "--prefix ""$workDir\website"" vite ""$workDir\website"" --host 127.0.0.1 --port 5174 --strictPort" -WindowStyle Hidden

# Verify Vite is up
for ($i = 0; $i -lt 20; $i++) {
    Start-Sleep -Seconds 1
    try {
        $res = Invoke-WebRequest -Uri "http://127.0.0.1:5174" -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if ($res.StatusCode -eq 200) { break }
    } catch {}
}

# Start Cloudflare Tunnel
$cfExe = "$env:TEMP\cloudflared.exe"
Start-Process -FilePath $cfExe -ArgumentList "tunnel --url http://127.0.0.1:5174" -RedirectStandardOutput $outLog -RedirectStandardError $logFile -WindowStyle Hidden

# Wait for tunnel URL
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Seconds 1
    if (Test-Path $logFile) {
        $m = Select-String -Path $logFile -Pattern 'https://[a-zA-Z0-9-]+\.trycloudflare\.com'
        if ($m) {
            $m.Matches[0].Value | Out-File -FilePath $urlFile -Encoding utf8
            break
        }
    }
}
