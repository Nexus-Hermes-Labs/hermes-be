pub mod server;

/// Generated protobuf types for auth service
pub mod proto {
    /// Auth service v1
    pub mod auth {
        pub mod v1 {
            tonic::include_proto!("auth.v1");
        }
    }

    /// User service v1 (client only, for calling user-service)
    pub mod user {
        pub mod v1 {
            tonic::include_proto!("user.v1");
        }
    }

    /// Common types v1
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}
