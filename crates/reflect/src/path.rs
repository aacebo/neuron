#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path(Vec<String>);

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Path {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn name(&self) -> &str {
        self.0.last().unwrap()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn push(mut self, part: &str) -> Self {
        self.0.push(part.to_string());
        self
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(
            value
                .split("::")
                .filter(|v| *v != "r#mod")
                .map(|v| v.trim().to_string())
                .collect::<Vec<_>>(),
        )
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(
            value
                .split("::")
                .filter(|v| *v != "r#mod")
                .map(|v| v.trim().to_string())
                .collect::<Vec<_>>(),
        )
    }
}

impl<const N: usize> From<[&str; N]> for Path {
    fn from(value: [&str; N]) -> Self {
        Self::from(value.join("::"))
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.join("::"))
    }
}

impl std::ops::Add<&Self> for Path {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        let mut next = self;

        for part in rhs.iter() {
            next = next.push(part);
        }

        next
    }
}
