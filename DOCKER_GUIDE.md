# 🐳 SecureForge / RAZE-CARVE — Docker Quickstart Guide

This project includes fully automated Docker configurations for running both the **Web Portal (with PS 149 Videos & Downloads Hub)** and the **Core Forensic Engine (GUI, TUI, and CLI)**.

---

## 🌐 Pull Directly from Docker Hub (No Build Required!)

Your friend can run this from anywhere with Docker installed:

```bash
# Pull the latest image
docker pull lonewolf175/raze-carve:latest

# Run the container
docker run -d --name raze-carve -p 5174:80 lonewolf175/raze-carve:latest
```
* **Portal URL:** [http://localhost:5174](http://localhost:5174)
* **Docker Hub Page:** [https://hub.docker.com/r/lonewolf175/raze-carve](https://hub.docker.com/r/lonewolf175/raze-carve)

---

## 🚀 1-Step Execution via Docker Compose

Run from the root directory:

### 1. Launch the Web Portal (Downloads, Demos & Showcase)
```bash
docker compose up -d website
```
* **URL:** [http://localhost:5174](http://localhost:5174)
* Runs the production Nginx Alpine container serving the updated UI, PS 149 operational videos, and downloads hub.

### 2. Launch the Desktop Forensic Web GUI
```bash
docker compose up -d gui
```
* **URL:** [http://localhost:7878](http://localhost:7878)
* Connects directly to hardware storage controllers via `/dev` pass-through.

### 3. Launch the Interactive Defense Terminal (Ratatui TUI)
```bash
docker compose run --rm tui
```
* Launches the 60 FPS interactive `sforge` curses console inside your terminal.

### 4. Execute One-Shot CLI Tasks
```bash
# Enumerate storage devices
docker compose run --rm cli info

# Run 23 subsystem diagnostics
docker compose run --rm cli test
```

---

## 🛠️ Standalone Docker Build Commands

If building individual images manually:

```bash
# Build & Run Website Container
cd website
docker build -t raze-carve:latest .
docker run -d -p 5174:80 --name raze-carve raze-carve:latest

# Build & Run Core Forensic Daemon
cd ..
docker build -t secureforge:latest .
docker run -d --privileged -v /dev:/dev -p 7878:7878 --name secureforge secureforge:latest gui
```
