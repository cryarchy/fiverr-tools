use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Selector(String);

impl Selector {
    pub fn new(selector: String) -> Self {
        Self(selector)
    }

    pub fn nth_child(&self, index: usize) -> Self {
        Self(format!("{}:nth-child({index})", self.0))
    }

    pub fn append(&self, selector: &str) -> Self {
        Self(format!("{} {selector}", self.0))
    }
}

impl Deref for Selector {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::selector::Selector;

    #[test]
    fn selector() {
        let selector: &str = &Selector::new("li".to_owned()).nth_child(2).append("a");
        assert_eq!(selector, "li:nth-child(2) a")
    }
}
