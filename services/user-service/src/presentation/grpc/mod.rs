pub mod client;
pub mod server;

/// Generated protobuf types for user service
pub mod proto {
    /// User service v1
    pub mod user {
        pub mod v1 {
            tonic::include_proto!("user.v1");
        }
    }

    /// Auth service v1 (client only, for calling auth-service)
    pub mod auth {
        pub mod v1 {
            tonic::include_proto!("auth.v1");
        }
    }

    /// Common types v1
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}
