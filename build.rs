use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Download the OpenAPI schema
    let schema_url = "https://developers.cloudflare.com/api/openapi.json";
    println!("cargo:warning=Downloading OpenAPI schema from {}", schema_url);

    let schema_content = reqwest::blocking::get(schema_url)?
        .text()?;

    let schema_path = out_dir.join("openapi.json");
    fs::write(&schema_path, &schema_content)?;

    // Parse and patch the schema to add missing operation IDs and simplify complex schemas
    let mut spec_value: serde_json::Value = serde_json::from_str(&schema_content)
        .map_err(|e| {
            eprintln!("Failed to parse schema JSON: {}", e);
            e
        })?;

    // Add missing operation IDs
    if let Some(paths) = spec_value.get_mut("paths").and_then(|p| p.as_object_mut()) {
        for (path_name, path_item) in paths.iter_mut() {
            if let Some(operations) = path_item.as_object_mut() {
                for (method, operation) in operations.iter_mut() {
                    // Skip non-operation fields like parameters, summary, etc.
                    if !["get", "put", "post", "delete", "options", "head", "patch", "trace"].contains(&method.as_str()) {
                        continue;
                    }

                    if let Some(op_obj) = operation.as_object_mut() {
                        // Only add operationId if it's missing
                        if !op_obj.contains_key("operationId") {
                            // Generate operation ID from method and path
                            let operation_id = format!("{}_{}",
                                method,
                                path_name
                                    .trim_start_matches('/')
                                    .replace('/', "_")
                                    .replace('{', "")
                                    .replace('}', "")
                                    .replace('-', "_")
                            );
                            op_obj.insert(
                                "operationId".to_string(),
                                serde_json::Value::String(operation_id)
                            );
                        }
                    }
                }
            }
        }
    }

    // Simplify schemas that use allOf with just one item (Progenitor doesn't handle this well)
    if let Some(components) = spec_value.get_mut("components") {
        if let Some(schemas) = components.get_mut("schemas").and_then(|s| s.as_object_mut()) {
            for schema in schemas.values_mut() {
                simplify_schema(schema);
            }
        }
    }

    // Save the patched schema for debugging
    let patched_schema_path = out_dir.join("openapi_patched.json");
    fs::write(&patched_schema_path, serde_json::to_string_pretty(&spec_value)?)?;

    // Convert the patched JSON back to OpenAPI spec
    let spec: openapiv3::OpenAPI = serde_json::from_value(spec_value.clone())
        .map_err(|e| {
            eprintln!("Failed to deserialize OpenAPI spec: {}", e);
            eprintln!("This is likely due to schema incompatibilities.");
            eprintln!("Patched schema saved at: {:?}", patched_schema_path);
            e
        })?;

    // Generate the client code with patched schema
    let mut generator = progenitor::Generator::default();

    let tokens = generator.generate_tokens(&spec)
        .map_err(|e| {
            eprintln!("Failed to generate code with Progenitor: {}", e);
            e
        })?;
    let generated_code = tokens.to_string();

    let output_file = out_dir.join("cloudflare_api.rs");
    fs::write(&output_file, generated_code)?;

    println!("cargo:warning=Generated API client at {:?}", output_file);

    Ok(())
}

fn simplify_schema(schema: &mut serde_json::Value) {
    // Fix invalid schema combinations - enum with string constraints
    if schema.get("enum").is_some() {
        if let Some(obj) = schema.as_object_mut() {
            // Remove string-specific constraints that don't make sense with enum
            obj.remove("maxLength");
            obj.remove("minLength");
            obj.remove("pattern");
            obj.remove("format");
            // Ensure type is set to string for enums (unless it's explicitly something else)
            if !obj.contains_key("type") {
                obj.insert("type".to_string(), serde_json::json!("string"));
            }
        }
    }

    // Handle allOf - merge all schemas into one
    if let Some(all_of) = schema.get("allOf").and_then(|a| a.as_array()).cloned() {
        let mut merged = serde_json::json!({
            "type": "object",
            "properties": {}
        });

        // Merge all allOf items
        for item in &all_of {
            merge_into(&mut merged, item);
        }

        // If we got something useful, replace the schema
        if merged.get("properties").and_then(|p| p.as_object()).map(|o| !o.is_empty()).unwrap_or(false) {
            *schema = merged;
        } else {
            // Fallback: use the first non-$ref schema or convert to generic object
            let first_concrete = all_of.iter()
                .find(|s| !s.get("$ref").is_some())
                .cloned()
                .unwrap_or(serde_json::json!({"type": "object"}));
            *schema = first_concrete;
        }

        // Continue processing the merged schema
        simplify_schema(schema);
        return;
    }

    // Handle oneOf/anyOf - just use the first option to keep it simple
    if let Some(one_of) = schema.get("oneOf").and_then(|o| o.as_array()).cloned() {
        if let Some(first) = one_of.first().cloned() {
            *schema = first;
            simplify_schema(schema);
            return;
        }
    }

    if let Some(any_of) = schema.get("anyOf").and_then(|o| o.as_array()).cloned() {
        if let Some(first) = any_of.first().cloned() {
            *schema = first;
            simplify_schema(schema);
            return;
        }
    }

    // Recursively process nested schemas
    if let Some(properties) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
        for prop in properties.values_mut() {
            simplify_schema(prop);
        }
    }

    if let Some(items) = schema.get_mut("items") {
        simplify_schema(items);
    }

    if let Some(additional) = schema.get_mut("additionalProperties") {
        if additional.is_object() {
            simplify_schema(additional);
        }
    }
}

fn merge_into(target: &mut serde_json::Value, source: &serde_json::Value) {
    // Skip $ref schemas - we can't merge them easily
    if source.get("$ref").is_some() {
        return;
    }

    // Merge properties
    if let Some(source_props) = source.get("properties").and_then(|p| p.as_object()) {
        if let Some(target_props) = target.get_mut("properties").and_then(|p| p.as_object_mut()) {
            for (key, value) in source_props {
                target_props.insert(key.clone(), value.clone());
            }
        }
    }

    // Merge required fields
    if let Some(source_required) = source.get("required").and_then(|r| r.as_array()) {
        let target_required = target
            .as_object_mut()
            .and_then(|o| o.entry("required").or_insert(serde_json::json!([])).as_array_mut());

        if let Some(target_req) = target_required {
            for item in source_required {
                if !target_req.contains(item) {
                    target_req.push(item.clone());
                }
            }
        }
    }

    // Copy other fields if not present
    if let Some(source_obj) = source.as_object() {
        if let Some(target_obj) = target.as_object_mut() {
            for (key, value) in source_obj {
                if key != "properties" && key != "required" && !target_obj.contains_key(key) {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }
}
