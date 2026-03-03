//! Build script: compiles chat-service Protocol Buffer definitions.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/service/chat/v1/chat_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
