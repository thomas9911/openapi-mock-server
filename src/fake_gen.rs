use fake::RngExt;
use fake::{Fake, faker};

use serde_json::{Value, json};

pub fn generate_from_schema(schema: &Value, field_name: Option<&str>) -> Value {
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => generate_string(schema, field_name),
        Some("integer") | Some("number") => generate_number(schema),
        Some("boolean") => Value::Bool(rand::rng().random_bool(0.5)),
        Some("array") => generate_array(schema, field_name),
        Some("object") => generate_object(schema),
        None => {
            // Could be a oneOf/anyOf/allOf or just untyped
            if let Some(one_of) = schema.get("oneOf").and_then(Value::as_array) {
                if let Some(first) = one_of.first() {
                    return generate_from_schema(first, field_name);
                }
            }
            if let Some(any_of) = schema.get("anyOf").and_then(Value::as_array) {
                if let Some(first) = any_of.first() {
                    return generate_from_schema(first, field_name);
                }
            }
            if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
                let mut merged = json!({});
                for sub in all_of {
                    if sub.get("type").is_some() || sub.get("properties").is_some() {
                        let generated = generate_from_schema(sub, field_name);
                        if let (Some(m), Some(g)) = (merged.as_object_mut(), generated.as_object())
                        {
                            for (k, v) in g {
                                m.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
                return merged;
            }
            // Has properties but no explicit type
            if schema.get("properties").is_some() {
                return generate_object(schema);
            }
            Value::Null
        }
        _ => Value::Null,
    }
}

fn generate_string(schema: &Value, field_name: Option<&str>) -> Value {
    if let Some(enum_vals) = schema.get("enum").and_then(Value::as_array) {
        if !enum_vals.is_empty() {
            let idx = rand::rng().random_range(0..enum_vals.len());
            return enum_vals[idx].clone();
        }
    }

    if let Some(fmt) = schema.get("format").and_then(Value::as_str) {
        return match fmt {
            "date-time" => Value::String(chrono::Utc::now().to_rfc3339()),
            "date" => Value::String(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            "email" => {
                let email: String = faker::internet::en::FreeEmail().fake();
                Value::String(email)
            }
            "uuid" => Value::String(uuid::Uuid::new_v4().to_string()),
            "uri" | "url" => {
                let domain: String = faker::internet::en::DomainSuffix().fake();
                let word: String = faker::lorem::en::Word().fake();
                Value::String(format!("https://{}.{}", word, domain))
            }
            "password" => Value::String("********".to_string()),
            _ => Value::String(generate_string_by_field_name(field_name)),
        };
    }

    Value::String(generate_string_by_field_name(field_name))
}

fn generate_string_by_field_name(field_name: Option<&str>) -> String {
    let name = field_name.unwrap_or("").to_lowercase();

    // Email patterns
    if name.contains("email") || name.contains("mail") {
        let email: String = faker::internet::en::FreeEmail().fake();
        return email;
    }

    // Name patterns
    if name == "firstname" || name == "first_name" || name == "givenname" || name == "given_name" {
        let n: String = faker::name::en::FirstName().fake();
        return n;
    }
    if name == "lastname"
        || name == "last_name"
        || name == "surname"
        || name == "familyname"
        || name == "family_name"
    {
        let n: String = faker::name::en::LastName().fake();
        return n;
    }
    if name == "fullname" || name == "full_name" || name == "displayname" || name == "display_name"
    {
        let n: String = faker::name::en::Name().fake();
        return n;
    }
    if name.ends_with("name") && (name.contains("user") || name.contains("person")) {
        let n: String = faker::name::en::Name().fake();
        return n;
    }
    // plain "name" falls through to word/lorem below — too generic for a person name

    // Phone patterns
    if name.contains("phone") || name.contains("mobile") || name.contains("tel") {
        let p: String = faker::phone_number::en::PhoneNumber().fake();
        return p;
    }

    // Address patterns
    if name.contains("street") || name == "address" || name == "address1" || name == "address_line"
    {
        let a: String = faker::address::en::StreetName().fake();
        return a;
    }
    if name.contains("city") {
        let c: String = faker::address::en::CityName().fake();
        return c;
    }
    if name.contains("country") {
        let c: String = faker::address::en::CountryName().fake();
        return c;
    }
    if name.contains("zip") || name.contains("postal") || name.contains("postcode") {
        let z: String = faker::address::en::ZipCode().fake();
        return z;
    }
    if name.contains("state") || name.contains("province") {
        let s: String = faker::address::en::StateName().fake();
        return s;
    }

    // URL/web patterns
    if name.contains("url") || name.contains("website") || name.contains("homepage") {
        let domain: String = faker::internet::en::DomainSuffix().fake();
        let word: String = faker::lorem::en::Word().fake();
        return format!("https://{}.{}", word, domain);
    }
    if name.contains("username") || name == "user_name" || name == "login" || name == "handle" {
        let u: String = faker::internet::en::Username().fake();
        return u;
    }

    // ID patterns
    if name == "id" || name.ends_with("_id") || name.ends_with("id") {
        return uuid::Uuid::new_v4().to_string();
    }

    // Description/text patterns
    if name.contains("description")
        || name.contains("bio")
        || name.contains("about")
        || name.contains("summary")
    {
        let s: String = faker::lorem::en::Sentence(5..10).fake();
        return s;
    }
    if name.contains("content")
        || name.contains("body")
        || name.contains("message")
        || name.contains("text")
        || name.contains("note")
    {
        let s: String = faker::lorem::en::Paragraph(1..3).fake();
        return s;
    }
    if name.contains("title") || name.contains("subject") || name.contains("heading") {
        let s: String = faker::lorem::en::Sentence(3..6).fake();
        return s;
    }

    // Company patterns
    if name.contains("company")
        || name.contains("organization")
        || name.contains("organisation")
        || name.contains("employer")
    {
        let c: String = faker::company::en::CompanyName().fake();
        return c;
    }

    // Color patterns
    if name.contains("color") || name.contains("colour") {
        let colors = [
            "red", "blue", "green", "yellow", "purple", "orange", "black", "white",
        ];
        let idx = rand::rng().random_range(0..colors.len());
        return colors[idx].to_string();
    }

    // Fallback: random word
    let word: String = faker::lorem::en::Word().fake();
    word
}

fn generate_number(schema: &Value) -> Value {
    let min = schema.get("minimum").and_then(Value::as_f64).unwrap_or(0.0);
    let max = schema
        .get("maximum")
        .and_then(Value::as_f64)
        .unwrap_or(1000.0);
    let is_integer = schema.get("type").and_then(Value::as_str) == Some("integer");

    if is_integer {
        let v = rand::rng().random_range(min as i64..=max as i64);
        Value::Number(v.into())
    } else {
        let v = rand::rng().random_range(min..max);
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .unwrap_or(Value::Number(0.into()))
    }
}

fn generate_array(schema: &Value, field_name: Option<&str>) -> Value {
    let items = schema.get("items");
    let min = schema.get("minItems").and_then(Value::as_u64).unwrap_or(1) as usize;
    let max = schema.get("maxItems").and_then(Value::as_u64).unwrap_or(3) as usize;
    let count = rand::rng().random_range(min..=max.max(min));

    let arr: Vec<Value> = (0..count)
        .map(|_| {
            items
                .map(|s| generate_from_schema(s, field_name))
                .unwrap_or(Value::Null)
        })
        .collect();

    Value::Array(arr)
}

fn generate_object(schema: &Value) -> Value {
    let mut map = serde_json::Map::new();

    if let Some(props) = schema.get("properties").and_then(Value::as_object) {
        for (key, prop_schema) in props {
            let value = generate_from_schema(prop_schema, Some(key));
            map.insert(key.clone(), value);
        }
    } else if let Some(additional) = schema.get("additionalProperties") {
        // e.g. { "type": "object", "additionalProperties": { "type": "integer" } }
        let keys = ["available", "pending", "sold"];
        for key in keys {
            map.insert(key.to_string(), generate_from_schema(additional, Some(key)));
        }
    }

    Value::Object(map)
}
