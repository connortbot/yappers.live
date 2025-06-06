#[derive(Debug, Clone)]
pub struct KeyBuilder {
    key: String,
}

impl KeyBuilder {
    pub fn new(base: impl ToString) -> Self {
        Self {
            key: base.to_string(),
        }
    }

    pub fn field(mut self, field: impl ToString) -> Self {
        self.key.push_str("::");
        self.key.push_str(&field.to_string());
        self
    }

    pub fn get_key(self) -> String {
        self.key
    }
}

impl ToString for KeyBuilder {
    fn to_string(&self) -> String {
        self.key.clone()
    }
}