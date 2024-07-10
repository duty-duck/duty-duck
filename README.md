# Uptime monitoring

## Dev environment setup

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