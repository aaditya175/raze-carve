$ErrorActionPreference = "Continue"

Write-Output "[*] Cleaning up old processes..."
Get-Process -Name cloudflared -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

$portConns = Get-NetTCPConnection -LocalPort 5174 -ErrorAction SilentlyContinue
foreach ($conn in $portConns) {
    if ($conn.OwningProcess -gt 0) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
    }
}
Start-Sleep -Seconds 1

Write-Output "[*] Starting Vite server on port 5174..."
$viteProc = Start-Process -FilePath "npx.cmd" -ArgumentList "--prefix website vite website --host 127.0.0.1 --port 5174 --strictPort" -PassThru

$ready = $false
for ($i = 0; $i -lt 15; $i++) {
    Start-Sleep -Seconds 1
    try {
        $res = Invoke-WebRequest -Uri "http://127.0.0.1:5174" -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if ($res.StatusCode -eq 200) {
            Write-Output "[*] Vite is UP and responding with HTTP 200!"
            $ready = $true
            break
        }
    } catch {}
}

if (-not $ready) {
    Write-Output "[!] Vite failed to start on 5174."
    exit 1
}

$cfExe = Join-Path $env:TEMP "cloudflared.exe"
Write-Output "[*] Starting Cloudflare Tunnel..."
& $cfExe tunnel --url http://127.0.0.1:5174
