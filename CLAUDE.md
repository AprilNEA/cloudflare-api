# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library that attempts to generate a Cloudflare API client from OpenAPI specifications using Progenitor. The project currently faces schema compatibility challenges due to the complexity of Cloudflare's official OpenAPI spec.

## Build System Architecture

### Code Generation at Build Time

The entire API client is generated during the build phase via [build.rs](build.rs):

1. **Schema Download**: Fetches OpenAPI spec from `https://developers.cloudflare.com/api/openapi.json`
2. **Schema Patching**: Applies automated fixes to make the schema compatible with Progenitor
3. **Code Generation**: Uses Progenitor to generate Rust client code into `OUT_DIR`
4. **Runtime Inclusion**: [src/lib.rs](src/lib.rs) includes the generated code via `include!` macro

The generated client code is not checked into version control - it's created fresh on every build.

### Schema Patching Logic

The [build.rs](build.rs) script performs several transformations:

- **Operation ID Generation**: Creates missing `operationId` fields from HTTP method + path
- **allOf Simplification**: Merges `allOf` compositions into single object schemas
- **oneOf/anyOf Resolution**: Takes first option to avoid complex type unions
- **Enum Validation Cleanup**: Removes conflicting string constraints from enum types
- **Recursive Processing**: Applies transformations to nested schemas

The patched schema is saved as `openapi_patched.json` in OUT_DIR for debugging.

## Commands

### Build the library
```bash
cargo build
```
This downloads the OpenAPI schema, patches it, generates client code, and compiles the library. Warning: build may fail due to schema complexity issues.

### Clean build artifacts
```bash
cargo clean
```
Removes all generated code and build artifacts. Next build will re-download and regenerate everything.

### Check without full build
```bash
cargo check
```
Faster than full build but still runs the code generation step.

## Known Issues

The Cloudflare OpenAPI schema has several incompatibilities with Progenitor:

1. Missing operation IDs (partially fixed by build script)
2. Complex `allOf` compositions that Progenitor's typify library doesn't support
3. Invalid schema combinations (enum + string validation constraints)
4. Type deserialization errors from structural issues

Even with patching, the schema may be too complex for successful generation. See README.md for alternative approaches.

## Dependencies

**Runtime dependencies:**
- `progenitor-client`: Runtime support for generated API clients
- `reqwest`: HTTP client with JSON support
- `serde`/`serde_json`: Serialization framework
- `schemars`: JSON Schema support with chrono features

**Build dependencies:**
- `progenitor`: OpenAPI code generator
- `reqwest`: For downloading schema (blocking mode)
- `openapiv3`: OpenAPI v3 data structures for parsing/manipulation

## Architecture Decisions

- **Edition 2024**: Uses latest Rust edition
- **Code Generation vs Manual**: Chose code generation for maintainability, though schema complexity makes this challenging
- **Build-time Schema Patching**: Automated fixes to avoid manual schema maintenance
- **Dual License**: MIT OR Apache-2.0 for maximum compatibility
