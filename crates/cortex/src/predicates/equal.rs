use crate::Predicate;

pub struct Equal {
    a: String,
    b: String,
}

impl Equal {
    pub fn new(a: impl Into<String>, b: impl Into<String>) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

impl Predicate for Equal {
    fn name(&self) -> &'static str {
        "equal"
    }

    fn invoke(&self, ctx: &crate::Context) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}
