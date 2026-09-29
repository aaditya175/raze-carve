$outFile = Join-Path $env:TEMP "cloudflared_website_out.log"
$errFile = Join-Path $env:TEMP "cloudflared_website_err.log"
if (Test-Path $outFile) { Remove-Item $outFile -Force }
if (Test-Path $errFile) { Remove-Item $errFile -Force }

# Kill existing cloudflared
Get-Process -Name cloudflared -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# Verify or launch Vite
$check = $null
try {
    $check = Invoke-WebRequest -Uri 'http://127.0.0.1:5174' -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
} catch {}

if (-not $check -or $check.StatusCode -ne 200) {
    Get-Process -Name node -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Process -FilePath "npx.cmd" -ArgumentList "--prefix website vite website --host 127.0.0.1 --port 5174" -WindowStyle Hidden
    Start-Sleep -Seconds 4
}

# Start Cloudflare Tunnel detached
$cfExe = Join-Path $env:TEMP "cloudflared.exe"
Start-Process -FilePath $cfExe -ArgumentList "tunnel --url http://127.0.0.1:5174" -RedirectStandardOutput $outFile -RedirectStandardError $errFile -WindowStyle Hidden

# Wait for tunnel URL
$url = $null
for ($i = 0; $i -lt 25; $i++) {
    Start-Sleep -Seconds 1
    if (Test-Path $errFile) {
        $match = Select-String -Path $errFile -Pattern 'https://[a-zA-Z0-9-]+\.trycloudflare\.com'
        if ($match) {
            $url = $match.Matches[0].Value
            break
        }
    }
}

Write-Output "TUNNEL_URL:$url"
