fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile auth-service proto (server only)
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/auth/v1/auth_service.proto"],
            &["../../proto"],
        )?;

    // Compile user-service proto (client only, for calling user-service)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../proto/service/user/v1/user_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
