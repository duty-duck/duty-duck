# Duty Duck 🦆

This repository contains the code for the Duty Duck uptime monitoring platform, its website, its API client, and its command-line utility.

## What is it ?

Duty Duck is a platform that allows you to monitor the uptime of your services, and the liveliness of your recurring tasks, such as periodic backups.

Its features (developed or planned) include:
  - 🌐 HTTP uptime monitoring, using a real browser
  - ✅ Flexible content assertions
  - 📅 Task monitoring, for one-shot and recurring tasks
  - ⚠️ Incident management features (incident tagging, comments, acknowledgments, escalation policies, vacation mode) (some implemented, some planned)
  - ✨ Similar incident detection, automatically-generated incident reports, and suggestions for fixes (to be implemented)
  - 📧 SMS, Push and E-mail notifications
  - 📡 Webhooks for custom integrations (to be implemented)
  - 👥 A multi-tenant architecture, built to enable multiple organizations on a single deployment
  - 🌍 A multi-language dashboard (English and French at the moment)
  - 📊 Status pages

## Architecture

The platform is composed of:
- A PostgreSQL database
- A Keycloak server
- A back-end server in Rust 🦀, which is the main component of the platform, providing the API and the business logic
- A Rust library to interact with the platform
- A command-line utility to interact with the platform
- A front-end in Vue.js 3 and Nuxt 🖖
- A headless browser service, used to ping your services
- A fake internet service, used to provide testing endpoints during development

It also depends on a few external services:
- An SMTP server, used to send e-mails
- A Firebase account, used to send push notifications
- An S3-compatible object storage, used to store HTTP responses and screenshots
- AWS SNS, used to send SMS notifications

The back-end server is completely stateless. It heavily relies on PostgresSQL's features, such as:
- Declarative partitioning to implement data retention policies
- Row-level security to isolate tenants (to be implemented)
- `SKIP LOCKED` to implement concurrent job queues for many features (periodic HTTP calls, tasks lifecycle, notifications ...)
- Partial indexes to enforce consistency rules (e.g. an endpoint can have multiple incidents, but only one ongoing incident at a time)

We follow the "ports and adapters" architecture: core domain logic and entities are implemented in `components/server/src/domain`, external services are abstracted away using Rust traits we call "ports", and implementations, i.e. adapters, are provided in `components/server/src/infrastructure`.

We also strive to use the [*typestate* pattern](https://cliffle.com/blog/rust-typestate/) whenever possible. This pattern models domain entities
as finite state machines, whose transitions can be verified at compile-time. It is a way to *make illegal states unrepresentable*™️ (to quote the linked article, it makes entities "easy to use correctly and impossible to use incorrectly"). 

We use this pattern heavily in the task monitoring feature: a Task can transition from `Pending` to `Running`, then from `Running` to `Completed` or `Failed`, but never from `Failed` to `Completed`. This is all enforced by the type system.

The platform is deployed using Terraform and Nix. [This article explains our deployment approach](https://guillaumebogard.dev/posts/declarative-server-management-with-nix/).

## Dev container

If you use Visual Studio Code, this project provides a ready-to-use devcontainer with all the dependencies available. Starting the devcontainer will automatically start all
the required services (Keycloak, Postgresql, Maildev, etc.), which are defined in the `docker-compose.yml` file.

## Manual Dev environment setup

Make sure to install:
- Docker (or Podman) and Docker Compose
- Cargo
- Node.js >= v18
- SQLX ClI (`cargo install sqlx-cli`)
- Node modules (`cd frontend && npm installs`)

Start the required services using `docker compose up -d --scale dev-container=0`. 
The `--scale dev-container=0` flag is used to prevent the devcontainer from starting, since you don't want to use the devcontainer.

Then you can:
- Start the back-end server (`cd components/server && cargo run`)
- Start the front-end server (`cd components/frontend && npm start`)
- Start everything in dev mode and watch changes `npm run dev`
- Create SQL migrations (`cd components/server && sqlx migrate add -r <name>`)
- Run SQL migrations (`cd components/server && sqlx migrate run`)
- Rollback SQL migrations (`cd components/server && sqlx migrate revert`)

## Releasing Docker images

Docker files in this project are meant to be built from the context of the workspace root. This is because the `frontend` image needs to `COPY` files located in the `server` directory.

### Building the frontend:

```shell
docker build -t ghcr.io/duty-duck/frontend:latest -f components/frontend/Dockerfile .
```

### Building the server:

In order to build the server image, you will need to save SQL query metadata so the `server` binary can be built without contacting a database.

```shell
$(cd components/server && cargo sqlx prepare)
docker build -t ghcr.io/duty-duck/server:latest -f components/server/Dockerfile .
```

### Building the browser:

```shell
docker build -t ghcr.io/duty-duck/browser:latest -f components/browser/Dockerfile .
```

### Building the "fake internet" service:

```shell
docker build -t ghcr.io/duty-duck/fake-internet:latest -f components/fake-internet/Dockerfile .
```

### Building the Keycloak image:

```shell
docker build -t ghcr.io/duty-duck/keycloak:latest -f components/keycloak/Dockerfile .
```

## Keycloak checklist

See [keycloak.md](docs/keycloak.md) for more information.