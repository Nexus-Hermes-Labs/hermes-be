fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile auth-service proto (server only)
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/auth/v1/auth_service.proto"],
            &["../../proto"],
        )?;

    // Compile user-service proto
    // Server stubs are also generated so integration tests can create mock gRPC servers.
    // They're dead code in production and stripped by the linker.
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../../proto/service/user/v1/user_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
