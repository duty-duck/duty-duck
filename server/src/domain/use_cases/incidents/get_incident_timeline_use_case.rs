use futures::{stream::FuturesOrdered, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident_event::IncidentEvent,
        user::User,
    },
    ports::{incident_event_repository::IncidentEventRepository, user_repository::UserRepository},
};

#[derive(Serialize, Deserialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetIncidentTimelineParams {
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
}

#[derive(Serialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetIncidentTimelineResponse {
    pub items: Vec<TimelineItem>,
}

#[derive(Serialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineItem {
    pub event: IncidentEvent,
    pub user: Option<User>,
}

#[derive(Error, Debug)]
pub enum GetIncidentTimelineError {
    #[error("Current user doesn't have the privilege the see incidents events")]
    Forbidden,
    #[error("Failed to get incidents from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_incident_timeline(
    auth_context: &AuthContext,
    incident_event_repository: &impl IncidentEventRepository,
    user_repository: &impl UserRepository,
    incident_id: Uuid,
    params: GetIncidentTimelineParams,
) -> anyhow::Result<GetIncidentTimelineResponse, GetIncidentTimelineError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(GetIncidentTimelineError::Forbidden);
    }
    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);

    let events = incident_event_repository
        .get_incident_timeline(
            auth_context.active_organization_id,
            incident_id,
            items_per_page,
            page_number,
        )
        .await?;

    let items = events
        .into_iter()
        .map(|event| async move {
            if let Some(user_id) = event.user_id {
                let user = user_repository.get_user(user_id, true).await.ok().flatten();
                TimelineItem { event, user }
            } else {
                TimelineItem { event, user: None }
            }
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await;

    Ok(GetIncidentTimelineResponse { items })
}
