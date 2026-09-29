<#
.SYNOPSIS
    Automated uploader for the 1.35 GB RAZE-CARVE Live ISO to GitHub Releases.
.DESCRIPTION
    Uses GitHub REST API to upload the ISO file directly to a specified release tag.
.PARAMETER GitHubToken
    GitHub Personal Access Token with 'repo' scope.
.PARAMETER RepoOwner
    GitHub repository owner (default: 'aaditya175').
.PARAMETER RepoName
    GitHub repository name (default: 'raze-carve').
.PARAMETER TagName
    Release tag name (default: 'v0.2.1').
#>
param (
    [Parameter(Mandatory = $false)]
    [string]$GitHubToken,

    [Parameter(Mandatory = $false)]
    [string]$RepoOwner = "aaditya175",

    [Parameter(Mandatory = $false)]
    [string]$RepoName = "raze-carve",

    [Parameter(Mandatory = $false)]
    [string]$TagName = "v0.2.1"
)

$isoPath = Join-Path $PSScriptRoot "..\website\public\releases\raze-carve-live-0.2.1-gui-amd64.iso"

if (-not (Test-Path $isoPath)) {
    Write-Error "ISO file not found at: $isoPath"
    exit 1
}

$fileItem = Get-Item $isoPath
$fileBytes = $fileItem.Length
Write-Host "Found ISO file: $($fileItem.Name) ($([math]::Round($fileBytes / 1GB, 2)) GB)" -ForegroundColor Cyan

if (-not $GitHubToken) {
    Write-Host "`nTo upload automatically via API, provide a GitHub Personal Access Token:" -ForegroundColor Yellow
    Write-Host "Usage: .\upload_to_github_release.ps1 -GitHubToken 'your_token_here'" -ForegroundColor Yellow
    Write-Host "`nAlternatively, upload via GitHub Web UI at:" -ForegroundColor Green
    Write-Host "https://github.com/$RepoOwner/$RepoName/releases/new" -ForegroundColor Green
    exit 0
}

$headers = @{
    "Authorization" = "token $GitHubToken"
    "Accept" = "application/vnd.github.v3+json"
}

Write-Host "Checking if release '$TagName' exists..." -ForegroundColor Cyan
$releaseUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/tags/$TagName"

try {
    $release = Invoke-RestMethod -Uri $releaseUrl -Headers $headers -Method Get
} catch {
    Write-Host "Release not found. Creating new release '$TagName'..." -ForegroundColor Yellow
    $createBody = @{
        tag_name = $TagName
        name = "RAZE-CARVE $TagName — Universal Live Bootable Forensic OS"
        body = "Automated release containing the 1.35 GB bootable ISO image."
        draft = $false
        prerelease = $false
    } | ConvertTo-Json

    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$RepoOwner/$RepoName/releases" -Headers $headers -Method Post -Body $createBody
}

$uploadUrlTemplate = $release.upload_url
$uploadUrl = $uploadUrlTemplate -replace '\{.*\}', "?name=$($fileItem.Name)"

Write-Host "Uploading 1.35 GB ISO to GitHub Releases ($uploadUrl)..." -ForegroundColor Cyan
Write-Host "This will take a few minutes depending on your internet connection..." -ForegroundColor Yellow

$uploadHeaders = @{
    "Authorization" = "token $GitHubToken"
    "Content-Type" = "application/octet-stream"
}

Invoke-RestMethod -Uri $uploadUrl -Headers $uploadHeaders -Method Post -InFile $isoPath

Write-Host "`nSuccessfully uploaded ISO to GitHub Releases!" -ForegroundColor Green
Write-Host "Direct Download URL: https://github.com/$RepoOwner/$RepoName/releases/download/$TagName/$($fileItem.Name)" -ForegroundColor Green
