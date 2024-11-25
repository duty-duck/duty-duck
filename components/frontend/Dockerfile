# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-frontend:latest -f frontend/Dockerfile . 
FROM node:18-alpine AS base
ENV NODE_ENV=production
WORKDIR /src

# Build
FROM base AS build

# Install dependencies in a seperate step for caching
COPY --link components/frontend/package.json components/frontend/package-lock.json ./
RUN npm install --production=false

COPY ./components/server/bindings ../server/bindings
COPY --link components/frontend .

RUN npm run build

# Run
FROM base
ENV PORT=3000
COPY --from=build /src/.output /src/.output
# Optional, only needed if you rely on unbundled dependencies
# COPY --from=build /src/node_modules /src/node_modules

CMD [ "node", ".output/server/index.mjs" ]