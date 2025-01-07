## Keycloak checklist

Until we implement a way to manage the Keycloak configuration declaratively, you will need to configure the Keycloak realm manually
every time you install keycloak with a fresh database.

- Create a `dutyduck-server` client with
    - Correct Redirect URIs
    - Client credentials grant type
    - The `realm_admin` role
- Create a `dutyduck-dashboard` client with
    - Correct Redirect URIs
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
- Make sure there is a `dutyduck-dashboard` client scope with an audience mapper to include the `dutyduck-dashboard` audience to the token,
and make sure this client scope is enabled for the `dutyduck-dashboard` client
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
- Make sure the "active_organization" and the "phone" client scopes are enabled for the "dutyduck-dashboard" client
- Check e-mail server configuration
- Check the theme configuration for the realm. To see the organization swticher, the admin theme phasetwo.v2 must be enabled **on the master realm**
- Make sure e-mail verification is enabled for the realm
- Make sure registration is disabled for the realm
- Test the sign up feature and editing the user's profile (e.g. phone number)