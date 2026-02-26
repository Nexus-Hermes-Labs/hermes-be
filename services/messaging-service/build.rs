//! Build script: compiles messaging-service Protocol Buffer definitions.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/messaging/v1/messaging_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
