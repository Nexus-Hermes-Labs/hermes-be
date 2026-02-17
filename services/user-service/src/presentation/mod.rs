//! Presentation Layer
//!
//! Contains HTTP REST API and gRPC interfaces.
//!
//! ## HTTP REST
//! - User-facing API
//! - DTOs for request/response
//! - Handlers for business logic orchestration
//! - Routes configuration
//!
//! ## gRPC
//! - Service-to-service communication
//! - Auth-service calls user-service to create profiles
//! - Other services can query user data

pub mod dto;
pub mod grpc;
pub mod http;
pub mod error;
