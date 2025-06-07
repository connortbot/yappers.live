use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeySchema {
    pub pattern: Vec<KeySegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeySegment {
    Fixed(&'static str),
    Field(&'static str),
}

pub struct KeySchemas {
    schemas: HashMap<&'static str, KeySchema>,
}

impl KeySchemas {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();
        
        schemas.insert("game_code", KeySchema {
            pattern: vec![
                KeySegment::Fixed("game_code"),
                KeySegment::Field("code"),
            ],
        });

        schemas.insert("player_to_game", KeySchema {
            pattern: vec![
                KeySegment::Fixed("player_to_game"),
                KeySegment::Field("player_id"),
            ],
        });
        
        schemas.insert("player_auth", KeySchema {
            pattern: vec![
                KeySegment::Fixed("player_auth"),
                KeySegment::Field("player_id"),
            ],
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
}

impl KeyBuilder {
    pub fn new(schema_name: &str) -> Result<Self, String> {
        let schemas = get_schemas();
        let schema = schemas.get(schema_name)
            .ok_or_else(|| format!("Unknown key schema: {}", schema_name))?;
            
        Ok(Self {
            schema,  // &'static KeySchema
            values: Vec::new(),
            current_segment: 0,
        })
    }
    
    pub fn field(mut self, value: impl ToString) -> Result<Self, String> {
        if self.current_segment >= self.schema.pattern.len() {
            return Err("Too many segments provided for schema".to_string());
        }
        
        match &self.schema.pattern[self.current_segment] {
            KeySegment::Fixed(fixed_val) => {
                self.values.push(fixed_val.to_string());
                self.current_segment += 1;
                
                if self.current_segment >= self.schema.pattern.len() {
                    return Err("No field expected after fixed segment".to_string());
                }
                
                match &self.schema.pattern[self.current_segment] {
                    KeySegment::Field(_) => {
                        self.values.push(value.to_string());
                        self.current_segment += 1;
                        Ok(self)
                    }
                    KeySegment::Fixed(_) => {
                        Err("Expected field but found fixed segment".to_string())
                    }
                }
            }
            KeySegment::Field(_) => {
                self.values.push(value.to_string());
                self.current_segment += 1;
                Ok(self)
            }
        }
    }
    
    pub fn get_key(mut self) -> Result<String, String> {
        while self.current_segment < self.schema.pattern.len() {
            match &self.schema.pattern[self.current_segment] {
                KeySegment::Fixed(fixed_val) => {
                    self.values.push(fixed_val.to_string());
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