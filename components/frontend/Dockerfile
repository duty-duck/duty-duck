# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-frontend:latest -f frontend/Dockerfile . 
FROM node:18-alpine AS base
ENV NODE_ENV=production
WORKDIR /app

# Build
FROM base AS build

# Install dependencies in a seperate step for caching
COPY components/frontend/package*.json  ./components/frontend/
COPY package*.json ./

WORKDIR /app/components/frontend
RUN npm install --production=false

COPY ./components/server/bindings ../server/bindings
COPY components/frontend .

RUN npm run build

# Run
FROM base
ENV PORT=3000
COPY --from=build /app/components/frontend/.output /app/components/frontend/.output
# Optional, only needed if you rely on unbundled dependencies
# COPY --from=build /src/node_modules /src/node_modules

CMD [ "node", "/app/components/frontend/.output/server/index.mjs" ]