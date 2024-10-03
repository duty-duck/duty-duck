use nanoid::nanoid;

use crate::{attributes, domain::entities::organization::Address};

use super::*;

async fn build_client() -> anyhow::Result<KeycloakClient> {
    let keycloak_url = Url::parse("https://auth.dutyduck.net")?;
    let client_id = "dutyduck-server";
    let client_secret = "a90CJPOQBafVQaUG8Iz8zUJh5agzJk3M";
    let keycloak_realm = "duty-duck-preprod";

    KeycloakClient::new(keycloak_url.clone(), keycloak_url, keycloak_realm, client_id, client_secret).await
}

#[tokio::test]
#[ignore]
async fn test_obtain_token() -> anyhow::Result<()> {
    let client = build_client().await?;
    let access_token = client.obtain_access_token().await;
    println!("{:#?}", access_token);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_refresh_token() -> anyhow::Result<()> {
    let client = build_client().await?;

    let access_token = client.obtain_access_token().await;
    let access_token = client.refresh_access_token(&access_token).await;
    println!("{:#?}", access_token);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_organizations() -> anyhow::Result<()> {
    let client = build_client().await?;
    let orgs = client.get_organizations(0, 10, "").await?;
    println!("{:#?}", orgs);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_create_organization() -> anyhow::Result<()> {
    let client = build_client().await?;
    let request = WriteOrganizationRequest {
        realm: &client.realm,
        name: format!("test-organization-{}", nanoid!(10)),
        display_name: "Test organization".to_string(),
        url: None,
        domains: vec![],
        attributes: attributes! {
            "stripe_customer_id".to_string() => vec![],
            "billing_address".to_string() => vec![serde_json::to_string(&Address {
                line_1: "Foo".to_string(),
                line_2: "Bar".to_string(),
                ..Default::default()
            }).unwrap()],
        },
    };
    client.create_organization(&request).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_create_organization_role() -> anyhow::Result<()> {
    let client = build_client().await?;
    let request = WriteOrganizationRequest {
        realm: &client.realm,
        name: format!("test-organization-{}", nanoid!(10)),
        display_name: "Test organization".to_string(),
        url: None,
        domains: vec![],
        attributes: AttributeMap::default(),
    };
    let org = client.create_organization(&request).await?;
    client
        .create_an_organization_role(org.id, "test role")
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_update_organiaztion() -> anyhow::Result<()> {
    let client = build_client().await?;
    let request = WriteOrganizationRequest {
        realm: &client.realm,
        name: format!("test-organization-{}", nanoid!(10)),
        display_name: "Test organization".to_string(),
        url: None,
        domains: vec![],
        attributes: attributes! {
            "foo".to_string() => vec!["bar".to_string()],
        },
    };
    let org = client.create_organization(&request).await?;

    let request = WriteOrganizationRequest {
        display_name: "Test organization (Updated)".to_string(),
        attributes: attributes! {
            "foo".to_string() => vec!["bar (updated)".to_string()],
            "baz".to_string() => vec!["qux".to_string()],
        },
        ..request
    };
    client.update_organization(org.id, &request).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_list_organization_members() -> anyhow::Result<()> {
    let client = build_client().await?;

    // Create org
    let request = WriteOrganizationRequest {
        realm: &client.realm,
        name: format!("test-organization-{}", nanoid!(10)),
        display_name: "Test organization".to_string(),
        url: None,
        domains: vec![],
        attributes: attributes! {
            "foo".to_string() => vec!["bar".to_string()],
        },
    };
    let org = client.create_organization(&request).await?;
    println!("created org {}", org.name);

    // create member
    let request = CreateUserRequest {
        email: Some("test-list-members-user@noreply.com".to_string()),
        enabled: true,
        email_verified: true,
        first_name: Some("Jane".to_string()),
        last_name: Some("Doe".to_string()),
        attributes: AttributeMap::default(),
        groups: vec![],
        credentials: vec![Credentials {
            credentials_type: CredentialsType::Password,
            value: "1234".to_string(),
            temporary: false,
        }],
    };
    let user = client.create_user(&request).await?;

    // add member
    client.add_an_organization_member(org.id, user.id).await?;

    // List members
    let response = client.list_organization_members(org.id, 0, 10).await?;

    println!("{:#?}", response);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_create_user() -> anyhow::Result<()> {
    let client = build_client().await?;
    let request = CreateUserRequest {
        email: Some("jane2@noreply.com".to_string()),
        enabled: true,
        email_verified: true,
        first_name: Some("Jane".to_string()),
        last_name: Some("Doe".to_string()),
        attributes: AttributeMap::default(),
        groups: vec![],
        credentials: vec![Credentials {
            credentials_type: CredentialsType::Password,
            value: "1234".to_string(),
            temporary: false,
        }],
    };
    client.create_user(&request).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_update_user() -> anyhow::Result<()> {
    let client = build_client().await?;
    let request = CreateUserRequest {
        email: Some("jane22@noreply.com".to_string()),
        enabled: true,
        email_verified: true,
        first_name: Some("Jane".to_string()),
        last_name: Some("Doe".to_string()),
        attributes: AttributeMap::default(),
        groups: vec![],
        credentials: vec![Credentials {
            credentials_type: CredentialsType::Password,
            value: "1234".to_string(),
            temporary: false,
        }],
    };
    let user = client.create_user(&request).await?;
    let user = client
        .update_user(
            user.id,
            &UpdateUserRequest {
                first_name: Some("UPDATED".to_string()),
                last_name: Some("UPDATED".to_string()),
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(user.first_name.as_deref(), Some("UPDATED"));
    assert_eq!(user.last_name.as_deref(), Some("UPDATED"));
    Ok(())
}