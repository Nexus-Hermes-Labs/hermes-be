//! Build script for channel-service: compiles Protocol Buffer definitions via tonic-build.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile channel-service proto (server only)
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/channel/v1/channel_service.proto"],
            &["../../proto"],
        )?;

    // Compile guild-service proto (client only, for verifying guilds and permissions)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../proto/service/guild/v1/guild_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
