# ============================================================================
# RAZE-CARVE / SecureForge Web Portal & Distribution Hub
# High-Performance Production Container (Nginx Alpine + Embedded ISO Release)
# ============================================================================

# Stage 1: Build React/Vite Frontend
FROM node:20-alpine AS builder

WORKDIR /app

# Install dependencies with caching
COPY package*.json ./
RUN npm install

# Copy source and static assets
COPY src ./src
COPY public/demo_videos ./public/demo_videos
COPY public/*.mp4 public/*.jpg public/*.png public/*.ico public/*.svg public/*.txt public/*.xml ./public/
COPY index.html tsconfig*.json vite.config.ts tailwind.config.js postcss.config.js ./

# Compile TypeScript and bundle production assets
RUN npm run build

# Stage 2: Minimal Production Nginx Web Server
FROM nginx:alpine

# Copy custom nginx configuration for SPA routing and gigabyte file streaming
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    client_max_body_size 2048M; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    location /demo_videos/ { \
        alias /usr/share/nginx/html/demo_videos/; \
        autoindex on; \
    } \
    location /releases/ { \
        alias /usr/share/nginx/html/releases/; \
        autoindex on; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Copy compiled web assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Ensure releases directory exists
RUN mkdir -p /usr/share/nginx/html/releases

# Copy production releases (including embedded ISO)
COPY public/releases /usr/share/nginx/html/releases

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
