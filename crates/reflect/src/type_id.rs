#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct TypeId(&'static str);

thread_local! {
    static INTERNED: std::cell::RefCell<std::collections::HashMap<String, &'static str>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}

impl TypeId {
    pub(crate) fn from_str(value: &'static str) -> Self {
        Self(value)
    }

    pub(crate) fn from_string(value: String) -> Self {
        let s = INTERNED.with(|m| {
            let mut map = m.borrow_mut();

            if let Some(&existing) = map.get(&value) {
                existing
            } else {
                let leaked: &'static str = Box::leak(value.clone().into_boxed_str());
                map.insert(value, leaked);
                leaked
            }
        });

        Self(s)
    }
}

impl std::fmt::Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for TypeId {}

impl PartialEq for TypeId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0) || self.0 == other.0
    }
}

impl PartialEq<&str> for TypeId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for TypeId {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

impl AsRef<TypeId> for TypeId {
    fn as_ref(&self) -> &TypeId {
        self
    }
}

impl AsMut<TypeId> for TypeId {
    fn as_mut(&mut self) -> &mut TypeId {
        self
    }
}

impl std::ops::Deref for TypeId {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl std::ops::DerefMut for TypeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}
