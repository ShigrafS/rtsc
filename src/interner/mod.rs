// String interner for identifier dedup.
// Avoids repeated heap allocations and string comparisons across parsing/binding.

use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NameId(pub u32);

impl NameId {
    pub const EMPTY: NameId = NameId(0);
}

#[derive(Debug)]
pub struct StringInterner {
    names: Vec<String>,
    map: HashMap<String, NameId>,
}

impl Default for StringInterner {
    fn default() -> Self {
        let mut interner = Self {
            names: Vec::with_capacity(1024),
            map: HashMap::with_capacity(1024),
        };
        // Reserve 0 for empty identifier
        interner.intern("");
        interner
    }
}

impl StringInterner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, s: &str) -> NameId {
        if let Some(&id) = self.map.get(s) {
            return id;
        }

        let id = NameId(self.names.len() as u32);
        let s_owned = s.to_string();
        self.names.push(s_owned.clone());
        self.map.insert(s_owned, id);
        id
    }

    pub fn resolve(&self, id: NameId) -> Option<&str> {
        self.names.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning_dedup() {
        let mut interner = StringInterner::new();
        let id1 = interner.intern("foo");
        let id2 = interner.intern("bar");
        let id3 = interner.intern("foo");

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(interner.resolve(id1), Some("foo"));
        assert_eq!(interner.resolve(id2), Some("bar"));
    }
}
