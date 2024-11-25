# Build stage
FROM node:18-slim AS builder

WORKDIR /app

# Copy package files
COPY components/browser/package*.json ./components/browser/

# Install dependencies
WORKDIR /app/components/browser
RUN npm ci

WORKDIR /app

# Copy source files and build scripts
COPY components/browser/ ./components/browser/
COPY protos/ ./protos/

WORKDIR /app/components/browser

# Make compile script executable and run it
RUN chmod +x ./compileProtos.sh && \
    ./compileProtos.sh

RUN npm run build

# Runtime stage
FROM node:18-slim AS runtime

# Install chrome
RUN apt-get update && apt-get install -y --no-install-recommends \
  fonts-liberation \
  libasound2 \
  libatk-bridge2.0-0 \
  libatk1.0-0 \
  libcups2 \
  libdrm2 \
  libgbm1 \
  libgtk-3-0 \
  libnspr4 \
  libnss3 \
  libx11-xcb1 \
  libxcomposite1 \
  libxdamage1 \
  libxrandr2 \
  xdg-utils \
  libu2f-udev \
  libxshmfence1 \
  libglu1-mesa \
  chromium \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app/browser

# Copy package files
COPY components/browser/package*.json ./

# Install production dependencies only
RUN npm ci --production

# Copy built files from builder
COPY --from=builder /app/components/browser/dist/ .

USER node

# Start the application
CMD ["node", "src/main.js"]