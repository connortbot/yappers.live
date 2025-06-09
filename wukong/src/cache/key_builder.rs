use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeySchema {
    pub base_pattern: Vec<KeySegment>,
    pub allowed_extensions: Vec<Vec<KeySegment>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeySegment {
    Fixed(Vec<&'static str>),
    Field(&'static str),
}

pub struct KeySchemas {
    schemas: HashMap<&'static str, KeySchema>,
}

impl KeySchemas {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();

        for (name, schema) in crate::game::types::get_key_schemas() {
            schemas.insert(name, schema);
        }

        for (name, schema) in crate::team_draft::types::get_key_schemas() {
            schemas.insert(name, schema);
        }

        schemas.insert("game_channel", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["game_channel"]),
                KeySegment::Field("game_id"),
            ],
            allowed_extensions: vec![],
        });
        
        schemas.insert("game_code", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["game_code"]),
                KeySegment::Field("code"),
            ],
            allowed_extensions: vec![],
        });

        schemas.insert("player_to_game", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["player_to_game"]),
                KeySegment::Field("player_id"),
            ],
            allowed_extensions: vec![],
        });
        
        schemas.insert("player_auth", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["player_auth"]),
                KeySegment::Field("player_id"),
            ],
            allowed_extensions: vec![],
        });
        
        Self { schemas }
    }
    
    pub fn get(&self, name: &str) -> Option<&KeySchema> {
        self.schemas.get(name)
    }
}

static SCHEMAS: std::sync::OnceLock<KeySchemas> = std::sync::OnceLock::new();

fn get_schemas() -> &'static KeySchemas {
    SCHEMAS.get_or_init(|| KeySchemas::new())
}

#[derive(Debug, Clone)]
pub struct KeyBuilder {
    schema: &'static KeySchema,
    values: Vec<String>,
    current_segment: usize,
    in_extensions: bool,
    current_extension_path: Option<usize>,
    extension_segment: usize,
}

impl KeyBuilder {
    pub fn new(schema_name: &str) -> Result<Self, String> {
        let schemas = get_schemas();
        let schema = schemas.get(schema_name)
            .ok_or_else(|| format!("Unknown key schema: {}", schema_name))?;
            
        Ok(Self {
            schema,
            values: Vec::new(),
            current_segment: 0,
            in_extensions: false,
            current_extension_path: None,
            extension_segment: 0,
        })
    }
    
    pub fn field(mut self, value: impl ToString) -> Result<Self, String> {
        let value_str = value.to_string();
        
        if self.in_extensions {
            if let Some(path_index) = self.current_extension_path {
                let extension_path = &self.schema.allowed_extensions[path_index];
                
                if self.extension_segment >= extension_path.len() {
                    return Err("Extension path is complete, no more segments allowed".to_string());
                }
                
                match &extension_path[self.extension_segment] {
                    KeySegment::Fixed(allowed_values) => {
                        if !allowed_values.contains(&value_str.as_str()) {
                            return Err(format!(
                                "Invalid fixed segment '{}'. Allowed values: {:?}", 
                                value_str, allowed_values
                            ));
                        }
                        self.values.push(value_str);
                        self.extension_segment += 1;
                        Ok(self)
                    }
                    KeySegment::Field(_) => {
                        self.values.push(value_str);
                        self.extension_segment += 1;
                        Ok(self)
                    }
                }
            } else {
                for (path_index, extension_path) in self.schema.allowed_extensions.iter().enumerate() {
                    if let Some(first_segment) = extension_path.first() {
                        match first_segment {
                            KeySegment::Fixed(allowed_values) => {
                                if allowed_values.contains(&value_str.as_str()) {
                                    self.current_extension_path = Some(path_index);
                                    self.values.push(value_str);
                                    self.extension_segment = 1;
                                    return Ok(self);
                                }
                            }
                            KeySegment::Field(_) => {
                                self.current_extension_path = Some(path_index);
                                self.values.push(value_str);
                                self.extension_segment = 1;
                                return Ok(self);
                            }
                        }
                    }
                }
                
                let available_options: Vec<String> = self.schema.allowed_extensions
                    .iter()
                    .filter_map(|path| {
                        path.first().and_then(|segment| match segment {
                            KeySegment::Fixed(values) => Some(format!("{:?}", values)),
                            KeySegment::Field(name) => Some(format!("Field({})", name)),
                        })
                    })
                    .collect();
                
                return Err(format!(
                    "No extension path matches '{}'. Available options: {}", 
                    value_str, available_options.join(", ")
                ));
            }
        }
        else {
            if self.current_segment >= self.schema.base_pattern.len() {
                if !self.schema.allowed_extensions.is_empty() {
                    self.in_extensions = true;
                    return self.field(value_str);
                } else {
                    return Err("No extensions allowed for this schema".to_string());
                }
            }
            
            match &self.schema.base_pattern[self.current_segment] {
                KeySegment::Fixed(allowed_values) => {
                    self.values.push(allowed_values[0].to_string());
                    self.current_segment += 1;
                    if self.current_segment < self.schema.base_pattern.len() {
                        return self.field(value_str);
                    } else if !self.schema.allowed_extensions.is_empty() {
                        self.in_extensions = true;
                        return self.field(value_str);
                    } else {
                        return Err("No field expected after base pattern".to_string());
                    }
                }
                KeySegment::Field(_) => {
                    self.values.push(value_str);
                    self.current_segment += 1;
                    Ok(self)
                }
            }
        }
    }
    
    pub fn get_key(mut self) -> Result<String, String> {
        while self.current_segment < self.schema.base_pattern.len() {
            match &self.schema.base_pattern[self.current_segment] {
                KeySegment::Fixed(allowed_values) => {
                    self.values.push(allowed_values[0].to_string());
                    self.current_segment += 1;
                }
                KeySegment::Field(field_name) => {
                    return Err(format!("Missing required field: {}", field_name));
                }
            }
        }
        
        Ok(self.values.join("::"))
    }
}

pub fn key(schema_name: &str) -> Result<KeyBuilder, String> {
    KeyBuilder::new(schema_name)
}

impl ToString for KeyBuilder {
    fn to_string(&self) -> String {
        self.clone().get_key().unwrap_or_else(|e| format!("Invalid key: {}", e))
    }
}