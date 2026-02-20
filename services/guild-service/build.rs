//! Build script for guild-service: compiles Protocol Buffer definitions via tonic-build.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile guild-service proto (server only)
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/guild/v1/guild_service.proto"],
            &["../../proto"],
        )?;

    // Compile user-service proto (client only, for fetching user profiles)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../proto/service/user/v1/user_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
