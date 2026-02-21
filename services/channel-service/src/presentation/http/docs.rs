#![allow(clippy::needless_for_each)]
use utoipa::OpenApi;

#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::channel::request::*;
#[allow(clippy::wildcard_imports)]
use crate::presentation::dto::channel::response::*;

#[allow(clippy::wildcard_imports)]
use crate::presentation::http::handlers::channel::*;
use common::observability::health::{
    __path_health_handler, __path_liveness_handler, __path_readiness_handler, ComponentHealth,
    HealthChecks, HealthResponse,
};

/// `OpenAPI` documentation for the channel service.
#[allow(clippy::needless_for_each)]
#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        create_channel,
        list_guild_channels,
        get_channel,
        update_channel,
        delete_channel,
        health_handler,
        liveness_handler,
        readiness_handler
    ),
    components(schemas(
        CreateChannelRequest,
        UpdateChannelRequest,
        ChannelResponse,
        ChannelListResponse,
        HealthResponse,
        HealthChecks,
        ComponentHealth
    )),
    tags(
        (name = "channels", description = "Channel management"),
        (name = "health", description = "Health check endpoints")
    ),
    info(
        title = "Channel Service API",
        version = "0.1.0",
        description = "Manages text, voice, category, and announcement channels within guilds"
    )
)]
pub struct ApiDoc;
