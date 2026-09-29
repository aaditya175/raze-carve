# 📦 Handling Large Files & 1.35 GB Live ISO on GitHub

This document explains how RAZE-CARVE handles large binary distribution assets (like the **1.35 GB Universal Live Bootable ISO**) within the GitHub and Docker ecosystems.

---

## ⚠️ Why Git Repositories Cannot Track 1.35 GB Files Directly

| Platform / Mechanism | Limit | Behavior with 1.35 GB File |
| :--- | :--- | :--- |
| **Standard Git Push** | **100 MB** hard cap | ❌ **Rejected**. GitHub terminates the push with error `GH001: Large files detected`. |
| **Git LFS (Free Tier)** | **1 GB** storage / 1 GB/mo bandwidth | ❌ **Fails**. A 1.35 GB file exceeds the 1 GB total storage quota and locks downloads after 1 download. |
| **GitHub Releases** | **2.0 GB** per file asset | ✅ **PERFECT & FREE**. Unlimited download bandwidth, direct permanent download URLs. |
| **Docker Hub Registry** | **No practical limit** (multi-GB layers) | ✅ **ACTIVE & LIVE**. Embedded in `lonewolf175/raze-carve:latest`. |

---

## 🎯 The 2 Official Distribution Channels for the 1.35 GB ISO

### Channel 1: GitHub Releases (Recommended for Direct Browser / Wget Downloads)

GitHub Releases allows you to attach binary files up to **2 GB each** for free on public repositories with **no bandwidth throttling**.

#### How to Publish the ISO to GitHub Releases:

1. **Push your code to GitHub first:**
   ```bash
   git remote add origin https://github.com/aaditya175/raze-carve.git
   git branch -M main
   git push -u origin main
   ```

2. **Create a Release on GitHub:**
   * Go to your repository on GitHub: `https://github.com/aaditya175/raze-carve`
   * Click **Releases** (on the right sidebar) or navigate to `https://github.com/aaditya175/raze-carve/releases/new`
   * In **Choose a tag**, enter: `v0.2.1` and click **Create new tag**
   * Set **Release title**: `RAZE-CARVE v0.2.1 — Universal Live Bootable Forensic OS`
   * In the description, paste release notes:
     ```markdown
     ### RAZE-CARVE Universal Live Bootable ISO (v0.2.1)
     * **Architecture:** x86_64 UEFI/BIOS Hybrid
     * **Kernel:** Linux LTS with NVMe, SAS, SATA, UFS, eMMC controller drivers
     * **Default Autologin:** `root` on tty1 into interactive `sforge` defense TUI
     * **SHA-256:** `dbcf2a4981ca7bdfb6dce6cd23a881e1b8431f52d63de089bb1e7ff50540af6e`
     ```
   * **Attach the 1.35 GB ISO:** Drag and drop the ISO file located at:
     ```
     website/public/releases/raze-carve-live-0.2.1-gui-amd64.iso
     ```
     into the **Attach binaries by dropping them here or selecting them** box.
   * Click **Publish release**.

3. **Permanent Public Download Link Generated:**
   Once published, anyone can download it directly from:
   ```
   https://github.com/aaditya175/raze-carve/releases/download/v0.2.1/raze-carve-live-0.2.1-gui-amd64.iso
   ```

---

### Channel 2: Docker Hub (Instant Extraction Anywhere)

Because the 1.35 GB ISO is already bundled inside the published Docker container `lonewolf175/raze-carve:latest`, any user can obtain it using standard Docker commands:

#### Option A: Stream and Extract Directly to a Local File (No Container Left Running)
```bash
docker run --rm lonewolf175/raze-carve:latest cat /usr/share/nginx/html/releases/raze-carve-live-0.2.1-gui-amd64.iso > raze-carve-live-0.2.1-gui-amd64.iso
```

#### Option B: Launch the Interactive Web Portal and Download from Browser
```bash
# Start the web portal
docker run -d --name raze-carve -p 5174:80 lonewolf175/raze-carve:latest

# Download via browser
http://localhost:5174/releases/raze-carve-live-0.2.1-gui-amd64.iso
```

---

## 🛡️ Verification After Download

Always verify the cryptographic integrity of the ISO before flashing to external media:

### Windows (PowerShell):
```powershell
Get-FileHash -Algorithm SHA256 raze-carve-live-0.2.1-gui-amd64.iso
# Expected Hash:
# DBCF2A4981CA7BDFB6DCE6CD23A881E1B8431F52D63DE089BB1E7FF50540AF6E
```

### Linux / macOS:
```bash
sha256sum raze-carve-live-0.2.1-gui-amd64.iso
```
