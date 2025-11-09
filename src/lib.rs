// Include the generated API client code
include!(concat!(env!("OUT_DIR"), "/cloudflare_api.rs"));

// Re-export commonly used types
pub use progenitor_client::{ByteStream, Error, ResponseValue};
