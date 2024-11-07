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

### Building the browser:

```shell
docker build -t duty-duck-browser:latest -f browser/Dockerfile .
```

## Keycloak checklist

- Create a client with `duty-duck-frontend` and `duty-duck-server` redirect URIs
- Create a client with `duty-duck-server` redirect URIs
- Make sure there is a Active_Organization_Info client scope with these mappers:
    - active_organization
        - token claim name: active_organization
        - claim JSON type: JSON
        - add to ID token: true
        - add to access token: true
        - add to userinfo: true
    - organization_roles
        - token claim name: organization_roles
        - claim JSON type: String
        - add to ID token: true
        - add to access token: true
        - add to userinfo: true
- Make sure these user attributes exist in the Realm settings:
    - phoneNumber
    - phoneNumberVerified
    - phoneNumberOtp
- Make sure there is a "phone" client scope with:
    - a "phone number" mapper
        - token claim name: phoneNumber (camelCase!)
        - Claim JSON type: String
    - a "phone number verified" mapper
        - token claim name: phoneNumberVerified
        - Claim JSON type: Boolean
    - a "phoneNumberOtp" mapper
        - token claim name: phoneNumberOtp
        - Claim JSON type: JSON
- Make sure the "active_organization" and the "phone" client scopes are enabled for the "duty-duck-dashboard" client
- Check e-mail server configuration
- Check the theme configuration for the realm. To see the organization swticher, the admin theme phasetwo.v2 must be enabled **on the master realm**
- Make sure e-mail verification is enabled for the realm
- Make sure registration is disabled for the realm
- Test the sign up feature and editing the user's profile (e.g. phone number)