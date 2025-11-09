# Cloudflare API Client (Progenitor-based)

A Rust client library for the Cloudflare API, generated from OpenAPI specifications using Progenitor.

## Status: Schema Compatibility Issues

Unfortunately, the official Cloudflare OpenAPI schema at `https://developers.cloudflare.com/api/openapi.json` has several compatibility issues with Progenitor:

1. **Missing Operation IDs**: Many operations lack `operationId` fields
2. **Complex `allOf` schemas**: The schema uses advanced `allOf` compositions that Progenitor's typify library doesn't support
3. **Invalid schema combinations**: Some schemas have `enum` with string validation constraints like `maxLength`
4. **Type errors**: The schema has some structural issues that prevent proper deserialization

This is a known limitation of Progenitor - it works best with well-formed, simpler OpenAPI specs.

## Alternative Approaches

### Option 1: Use the Official Cloudflare Rust SDK

The recommended approach is to use the official Cloudflare SDK:

```toml
[dependencies]
cloudflare = "0.10"
```

See: https://github.com/cloudflare/cloudflare-rs

### Option 2: Manual Schema Cleanup

Download and manually clean up the OpenAPI schema:

```bash
curl https://developers.cloudflare.com/api/openapi.json -o openapi.json
```

Then edit [openapi.json](openapi.json) to:
- Add missing `operationId` fields to all operations
- Replace complex `allOf` with simple object schemas
- Remove conflicting validation rules from enums
- Fix any structural issues

Then update [build.rs](build.rs:12) to use the local file:

```rust
// Instead of downloading:
let schema_content = fs::read_to_string("openapi.json")?;
```

### Option 3: Use a Simpler OpenAPI Generator

Try using `openapi-generator` or `swagger-codegen` which may handle the Cloudflare schema better:

```bash
# Using openapi-generator
openapi-generator generate \
  -i https://developers.cloudflare.com/api/openapi.json \
  -g rust \
  -o ./generated
```

### Option 4: Subset of the API

Create a custom OpenAPI spec with just the Cloudflare endpoints you need. This is often the most practical approach for large APIs.

## Project Structure

```
cloudflare-api/
├── Cargo.toml          # Dependencies and metadata
├── build.rs            # Build script that generates the client
├── src/
│   └── lib.rs          # Library entry point that includes generated code
└── README.md           # This file
```

## How the Build Process Works

1. **Download Schema** ([build.rs:12](build.rs#L12)): Fetches the OpenAPI JSON from Cloudflare
2. **Patch Schema** ([build.rs:21-62](build.rs#L21-L62)): Adds missing operation IDs and simplifies complex schemas
3. **Generate Code** ([build.rs:71-75](build.rs#L71-L75)): Uses Progenitor to generate Rust client code
4. **Include Generated Code** ([src/lib.rs:2](src/lib.rs#L2)): The library includes the generated code at compile time

## Schema Patching Logic

The [build.rs](build.rs) script attempts several fixes:

- Generates operation IDs from HTTP method + path
- Merges `allOf` schemas into single objects
- Simplifies `oneOf`/`anyOf` by taking the first option
- Removes invalid constraint combinations from enums
- Resolves nested schema compositions

However, the Cloudflare schema is too complex for these automated patches to fully resolve.

## Dependencies

- **progenitor** (0.8): OpenAPI code generator
- **progenitor-client** (0.8): Runtime support for generated clients
- **reqwest** (0.12): HTTP client
- **serde** (1.0): Serialization framework
- **openapiv3** (2.0): OpenAPI v3 data structures

## Future Work

If you want to make this work:

1. Extract a manageable subset of the Cloudflare API
2. Manually create a clean OpenAPI 3.0 spec for that subset
3. Add comprehensive tests for the generated client
4. Consider contributing patches to Progenitor to better handle complex schemas

## Example Usage (If it worked)

```rust
use cloudflare_api::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("https://api.cloudflare.com/client/v4");

    // Example: List zones
    // let zones = client.get_accounts_account_id_zones("your-account-id").await?;

    Ok(())
}
```

## License

This is a demonstration project. Check the Cloudflare API terms of service for API usage restrictions.
