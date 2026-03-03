// Re-export generated protobuf types for the chat service.
#[allow(
    clippy::wildcard_imports,
    clippy::doc_markdown,
    clippy::used_underscore_binding,
    clippy::used_underscore_items,
    clippy::unwrap_used,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::similar_names,
    clippy::default_trait_access,
    clippy::large_enum_variant,
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::struct_excessive_bools
)]
pub mod chat {
    pub mod v1 {
        tonic::include_proto!("chat.v1");
    }
}
