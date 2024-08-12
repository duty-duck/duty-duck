# Uptime monitoring

## Dev container

If you want to use Visual Studio Code, this project provides a ready-to-use devcontainer with all the dependencies available.

## Manual Dev environment setup

Make sure to install:
- Cargo
- Node.js >= v18
- SQLX ClI (`cargo install sqlx-cli`)
- Node modules (`cd frontend && npm installs`)

Then you can:
- Start the back-end server (`cd server && cargo run`)
- Start the front-end server (`cd frontend && npm start`)
- Start everything in dev mode and watch changes `npm run dev`
- Create SQL migrations (`cd server && sqlx migrate add -r <name>`)
- Run SQL migrations (`cd server && sqlx migrate run`)
- Rollback SQL migrations (`cd server && sqlx migrate revert`)

## Releasing Docker images

Docker files in this project are meant to be built from the context of the workspace root. This is because the `frontend` image needs to `COPY` files located in the `server` directory.

### Building the frontend:

```shell
docker build -t duty-duck-frontend:latest -f frontend/Dockerfile .
```

### Building the server:

In order to build the server image, you will need to save SQL query metadata so the `server` binary can be built without contacting a database.

```shell
$(cd server && cargo sqlx prepare)
docker build -t duty-duck-server:latest -f server/Dockerfile .
```