# Build stage
FROM node:23-slim AS builder

WORKDIR /app

# Copy package files
COPY package-lock.json package.json /app/
COPY components/browser/package.json /app/components/browser/

# Install dependencies
RUN npm ci -w components/browser

# Copy source files and build scripts
COPY components/browser/ /app/components/browser/
COPY protos/ /app/protos/

WORKDIR /app/components/browser

# Make compile script executable and run it
RUN chmod +x ./compileProtos.sh && \
    ./compileProtos.sh

RUN npm run build

# Runtime stage
FROM node:18-slim AS runtime

# Install chrome
RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  fonts-liberation \
  libasound2 \
  libatk-bridge2.0-0 \
  libatk1.0-0 \
  libc6 \
  libcairo2 \
  libcups2 \
  libdbus-1-3 \
  libexpat1 \
  libfontconfig1 \
  libgbm1 \
  libgcc1 \
  libglib2.0-0 \
  libgtk-3-0 \
  libnspr4 \
  libnss3 \
  libpango-1.0-0 \
  libpangocairo-1.0-0 \
  libstdc++6 \
  libx11-6 \
  libx11-xcb1 \
  libxcb1 \
  libxcomposite1 \
  libxcursor1 \
  libxdamage1 \
  libxext6 \
  libxfixes3 \
  libxi6 \
  libxrandr2 \
  libxrender1 \
  libxss1 \
  libxtst6 \
  lsb-release \
  wget \
  xdg-utils \
  chromium \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Copy package files
COPY package-lock.json package.json /app/
COPY components/browser/package.json /app/components/browser/

# Install production dependencies only
RUN npm ci -w components/browser --production

WORKDIR /app/components/browser

# Copy built files from builder
COPY --from=builder /app/components/browser/dist/ .

USER node

# Start the application
CMD ["node", "src/main.js"]