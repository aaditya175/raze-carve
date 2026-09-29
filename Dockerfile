# ============================================================================
# Stage 1: Build React Web GUI Frontend
# ============================================================================
FROM node:20-bookworm-slim AS frontend-builder

WORKDIR /app
COPY src-ui/package*.json ./
RUN npm install
COPY src-ui/ ./
RUN npm run build

# ============================================================================
# Stage 2: Build Rust Binaries (CLI, TUI, Desktop/Web Server)
# ============================================================================
FROM rust:bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libsqlite3-dev \
    libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
    librsvg2-dev patchelf \
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY src-tauri/ src-tauri/
COPY plugins/ plugins/
COPY --from=frontend-builder /app/dist /build/src-ui/dist

RUN cargo build --release -p sih149-cli -p sih149-tui -p secureforge-desktop 2>&1

# ============================================================================
# Stage 3: Minimal Universal Runtime (Linux, Windows Docker Desktop, macOS)
# ============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates smartmontools util-linux hdparm nvme-cli \
    libgtk-3-0 libwebkit2gtk-4.1-0 libsoup-3.0-0 libjavascriptcoregtk-4.1-0 libsqlite3-0 \
    libpango-1.0-0 libharfbuzz0b libpangoft2-1.0-0 \
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=rust-builder /build/target/release/sih149 /usr/local/bin/sih149
COPY --from=rust-builder /build/target/release/sforge /usr/local/bin/sforge
COPY --from=rust-builder /build/target/release/secureforge-desktop /usr/local/bin/secureforge-desktop
RUN ln -s /usr/local/bin/sih149 /usr/local/bin/secureforge

# Copy static frontend web UI bundle for browser access
COPY --from=frontend-builder /app/dist /usr/local/share/secureforge/dist

# Copy plugins and Python report generation pipeline
COPY plugins/ /usr/local/share/secureforge/plugins/
COPY pipeline/ /usr/local/share/secureforge/pipeline/

# Setup isolated Python virtualenv for forensics pipeline
RUN python3 -m venv /opt/secureforge-venv && \
    /opt/secureforge-venv/bin/pip install --no-cache-dir \
    jinja2 weasyprint rfc3161ng pyOpenSSL

ENV PATH="/opt/secureforge-venv/bin:$PATH"
ENV SECUREFORGE_PLUGINS_DIR=/usr/local/share/secureforge/plugins
ENV SECUREFORGE_PIPELINE_DIR=/usr/local/share/secureforge/pipeline

# Terminal configuration for Ratatui TUI
ENV TERM=xterm-256color

# Universal entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

LABEL org.opencontainers.image.title="SecureForge"
LABEL org.opencontainers.image.description="Universal Forensic Data Sanitization, Recovery & Certification Station"
LABEL org.opencontainers.image.version="0.2.1"

WORKDIR /data
EXPOSE 7878

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["--help"]
