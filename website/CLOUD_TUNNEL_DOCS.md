# SecureForge Web Portal & Cloud Tunneling Technical Documentation

Comprehensive technical reference detailing the codebase architecture, development server mechanics, Cloudflare reverse-tunnel networking, and runtime troubleshooting.

---

## 1. System Overview & Repository Structure

The SecureForge codebase contains two distinct UI implementations:
- `src-ui/`: The desktop forensic application UI (built for Tauri).
- `website/`: The standalone web portal, documentation hub, downloads center, and browser-based virtualization sandbox.

### 1.1 Directory Breakdown of `website/`

```text
website/
├── index.html              # Main HTML entry point (loads fonts, v86 emulator, and React root)
├── docs.html               # Standalone static documentation page
├── downloads.html          # Release binaries and cryptographic checksums page
├── package.json            # NPM dependencies and script definitions
├── tsconfig.json           # TypeScript compiler configuration
├── vite.config.ts          # Vite bundler and development server configuration
├── tailwind.config.js      # Utility CSS theme definitions (colors, fonts, animations)
├── postcss.config.js       # CSS post-processing pipeline
├── public/                 # Static assets served as-is (favicons, images, wasm)
│   └── v86/                # x86 PC emulator JavaScript and WASM runtime
├── css/                    # Static styling stylesheets
├── js/                     # Standalone helper scripts
└── src/                    # Primary React application source code
    ├── main.tsx            # Application bootstrapping & DOM mounting
    ├── App.tsx             # Root React component, routing & view switching
    ├── index.css           # Global Tailwind directives and custom component classes
    ├── types.ts            # TypeScript interfaces and data models
    ├── components/         # Reusable UI elements (Navigation, Footer, Cards, Terminal)
    ├── pages/              # Page views (Home, Documentation, Features, Sandbox)
    ├── hooks/              # Custom React stateful hooks
    └── data/               # Static dataset definitions (command references, release data)
```

### 1.2 Key Application Features
1. **Interactive x86 Terminal Sandbox (`v86`)**:
   - Uses `public/v86/libv86.js` to run an in-browser WebAssembly-based x86 emulator.
   - Allows users to simulate running the SecureForge CLI (`sforge`) directly in the browser without installing native binaries.
2. **Dynamic Client-Side Routing**:
   - Managed in `src/App.tsx`, swapping active views (Home, Docs, Commands, Sandbox) without full page reloads.
3. **Hardware & Cryptographic Specifications Display**:
   - Outlines compliance standards (NIST SP 800-88 Rev 1, DoD 5220.22-M) and cryptographic algorithms (Shannon entropy calculation, SHA-256/512 audit logging).

---

## 2. Local Server Mechanics (Vite)

### 2.1 How Vite Operates
Unlike traditional bundlers (Webpack) that bundle the entire application into memory before serving, Vite uses **native ES Modules (ESM)**:
- In development, the browser requests modules individually as standard ES imports (e.g., `import React from 'react'`).
- Vite intercepts these HTTP requests, compiles TypeScript and JSX on-the-fly using `esbuild` (written in Go, 10-100x faster than JavaScript-based bundlers), and serves them instantly.
- **Hot Module Replacement (HMR)**: Vite establishes a WebSocket connection between the browser and the development server. When a source file is edited, only the modified module is pushed to the browser without a full page refresh.

### 2.2 Network Binding & Port Allocation
In `website/package.json`:
```json
"scripts": {
  "dev": "vite --port 5174",
  "build": "tsc && vite build",
  "preview": "vite preview --port 5174"
}
```
- **Port**: Default configured to `5174`.
- **IP Binding**: By default, Vite binds to `localhost` (`127.0.0.1` and `::1`).
- **Path Resolution**: When launching Vite from a parent directory, Vite must be explicitly instructed where the project root resides (`vite website`) so it can locate `website/index.html`.

---

## 3. Cloudflare Tunnel Networking Architecture

Cloudflare Tunnel (`cloudflared`) exposes a local port to the internet securely without port forwarding or dynamic DNS.

### 3.1 Traditional Port Forwarding vs. Cloudflare Tunnel

| Feature | Traditional Port Forwarding | Cloudflare Quick Tunnel |
| :--- | :--- | :--- |
| **Traffic Direction** | Inbound from Internet to Router | **Outbound only** from host to Cloudflare |
| **Router Configuration** | Manual NAT/Port Forwarding rules required | **Zero configuration**; traverses NAT/firewalls |
| **Public IP Exposure** | Your home/office public IP is exposed | **Hidden** behind Cloudflare Anycast IP |
| **SSL/TLS Certificates** | Manual setup (Let's Encrypt / Certbot) | **Automatic** TLS 1.3 edge certificates |
| **DDoS Protection** | None (direct router target) | **Built-in** Cloudflare Layer 3/4/7 protection |

### 3.2 Network Traffic Flow Diagram

```text
[ External User Browser / Mobile Device ]
                     │  (HTTPS on Port 443)
                     ▼
       [ Cloudflare Anycast Edge ]
       (e.g., Datacenter: bom12 - Mumbai)
       - Terminates SSL/TLS
       - Inspects for DDoS & malicious traffic
       - Maps domain (e.g., *.trycloudflare.com) to active tunnel session
                     │
                     │  (Encrypted QUIC / UDP or HTTP/2 Multiplexed Tunnel)
                     ▼
          [ Local Machine: cloudflared.exe ]
       - Maintains persistent outbound connection to Cloudflare edge
       - Decrypts tunnel packets
       - Rewrites target destination to localhost
                     │
                     │  (HTTP on Loopback: 127.0.0.1:5174)
                     ▼
            [ Vite Development Server ]
       - Handles request
       - Serves compiled React/TypeScript components
```

### 3.3 The `cloudflared` Lifecycle & Pre-Checks
When `cloudflared tunnel --url http://127.0.0.1:5174` executes:
1. **DNS Resolution**: Resolves `region1.v2.argotunnel.com` and `region2.v2.argotunnel.com`.
2. **UDP Connectivity (QUIC)**: Tests whether UDP traffic can reach Cloudflare on port 7844/443. QUIC is preferred due to low latency, multiplexing, and zero head-of-line blocking.
3. **TCP Connectivity (Fallback)**: Verifies HTTP/2 over TCP in case UDP is filtered by intermediate firewalls.
4. **API Authentication & Registration**: Communicates with `api.cloudflare.com` to register an ephemeral Quick Tunnel and obtain a unique subdomain (e.g., `https://events-basic-creation-brad.trycloudflare.com`).
5. **Edge Connection**: Establishes connections to nearest Cloudflare Points of Presence (e.g., `bom09`, `bom12` in Mumbai).

---

## 4. Complete Execution Walkthrough

### Step 1: Install Dependencies
```powershell
npm --prefix website install
```
Installs React, Tailwind CSS, TypeScript, and Vite into `website/node_modules/`.

### Step 2: Start the Vite Server
```powershell
npx --prefix website vite website --host 127.0.0.1 --port 5174 --strictPort
```
- `--prefix website`: Executes the local Vite binary inside `website/node_modules/.bin/`.
- `website`: Specifies the project root so `website/index.html` is served correctly.
- `--host 127.0.0.1`: Explicitly binds to IPv4 loopback.
- `--port 5174`: Enforces port 5174.
- `--strictPort`: Exits with an error if 5174 is in use rather than auto-switching to another port.

### Step 3: Run Cloudflare Tunnel
```powershell
& "$env:TEMP\cloudflared.exe" tunnel --url http://localhost:5174
```
Extracts the generated URL from the terminal output:
```text
+--------------------------------------------------------------------------------------------+
|  Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):  |
|  https://xxxxxxxx-xxxx-xxxx.trycloudflare.com                                              |
+--------------------------------------------------------------------------------------------+
```

---

## 5. Technical Troubleshooting & Post-Mortem Analysis

### 5.1 Error: 502 Bad Gateway (Host Error)
* **Root Cause**:
  1. A background process had already occupied port `5174`.
  2. Vite, launched without `--strictPort`, detected the busy port and auto-shifted to `5175`:
     ```text
     Port 5174 is in use, trying another one...
     ➜ Local: http://localhost:5175/
     ```
  3. `cloudflared` was configured to proxy to `http://127.0.0.1:5174`.
  4. When incoming requests reached Cloudflare Edge and forwarded over the tunnel to `127.0.0.1:5174`, the OS rejected the connection (`ECONNREFUSED`).
  5. Cloudflare translated this connection failure into an HTTP 502 Bad Gateway error page.
* **Fix**:
  - Kill conflicting processes on port 5174:
    ```powershell
    Get-NetTCPConnection -LocalPort 5174 -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
    ```
  - Use `--strictPort` to prevent automatic port shifting.

### 5.2 Error: 404 Not Found on Root URL
* **Root Cause**:
  Executing `npx --prefix website vite` from the parent repository directory without providing a target path defaults the Vite root directory to the parent folder (`SecureForge-main/`), which did not contain an `index.html`.
* **Fix**:
  Pass the directory explicitly: `npx --prefix website vite website`.

### 5.3 Error: CLI Parameter Failure with `untun`
* **Root Cause**:
  Executing `npx untun tunnel --port 5174` failed silently because `untun` expects a positional argument (`npx untun tunnel 5174`) rather than a `--port` flag.
* **Fix**:
  Standardized on the official `cloudflared.exe` binary.

---

## 6. Verification Checklist

To confirm the entire stack is operational:

1. **Verify Local Process**:
   ```powershell
   Get-NetTCPConnection -LocalPort 5174
   # Must show State: Listen, OwningProcess: <PID of node>
   ```
2. **Verify Local HTTP Response**:
   ```powershell
   Invoke-WebRequest -Uri "http://127.0.0.1:5174" -UseBasicParsing
   # Must return StatusCode: 200
   ```
3. **Verify Tunnel Connection**:
   Check `cloudflared` terminal output for:
   ```text
   Registered tunnel connection ... protocol=quic
   ```
4. **Verify Remote HTTPS Response**:
   ```powershell
   Invoke-WebRequest -Uri "https://<your-subdomain>.trycloudflare.com" -UseBasicParsing
   # Must return StatusCode: 200
   ```
