# This dockerfile is meant to be built from the context of the parent folder
# e.g.: docker build -t duty-duck-frontend:latest -f frontend/Dockerfile . 
FROM node:20 AS base
ENV NODE_ENV=production
RUN npm i --location=global pnpm
WORKDIR /app

# Build
FROM base AS build

# Install Playwirght
RUN pnpm install playwright
RUN npx playwright install --with-deps chromium

# Install dependencies in a seperate step for caching
COPY components/frontend/package*.json  ./components/frontend/
COPY package.json ./
COPY pnpm*.yaml ./

# Switch the frontend directory
WORKDIR /app/components/frontend
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install

COPY ./components/server/bindings ../server/bindings
COPY components/frontend .

# Build the app
RUN pnpm run build

# Run
FROM base
ENV PORT=3000
COPY --from=build /app/components/frontend/.output /app/components/frontend/.output

CMD [ "node", "/app/components/frontend/.output/server/index.mjs" ]