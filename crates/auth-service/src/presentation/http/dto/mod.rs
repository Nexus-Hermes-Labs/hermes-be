//! Application DTOs (Data Transfer Objects)
//!
//! This module contains all request and response types for the auth service HTTP API.
//!
//! ## Design Pattern: Type Aliases (DRY)
//!
//! Response types use type aliases to avoid code duplication:
//! - `LoginResponse` = `AuthResponse` (tokens only)
//! - `RefreshTokenResponse` = `AuthResponse` (tokens only)
//! - `RegisterResponse` = `AuthResponseWithUser` (tokens + user profile)
//!
//! This ensures consistency across endpoints while maintaining explicit types.

// ============================================
// MODULE DECLARATIONS
// ============================================

mod login;
mod logout;
mod refresh;
mod register;
mod shared;

// ============================================
// PUBLIC EXPORTS
// ============================================

// Shared types (base implementations)
pub use shared::{AuthResponse, AuthResponseWithUser, ClientInfo, UserProfile};

// Login endpoint
pub use login::{LoginRequest, LoginResponse};

// Logout endpoint
pub use logout::{LogoutRequest, LogoutResponse};

// Refresh endpoint
pub use refresh::{RefreshTokenRequest, RefreshTokenResponse};

// Register endpoint
pub use register::{RegisterRequest, RegisterResponse};
