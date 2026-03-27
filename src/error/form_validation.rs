use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct FormValidation(pub HashMap<String, Option<String>>);

impl FormValidation {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl std::fmt::Display for FormValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Form Validation:")?;
        for (k, v) in self.0.iter() {
            if let Some(reason) = v {
                writeln!(f, "> {k} : {reason}")?;
            } else {
                writeln!(f, "> {k}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for FormValidation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
