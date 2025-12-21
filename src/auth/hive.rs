use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct HivePermission {
    pub id: String,
    pub scope: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HivePermissionSet {
    set: HashMap<String, Option<String>>,
}

impl HivePermissionSet {
    pub fn new(set: HashMap<String, Option<String>>) -> Self {
        Self { set }
    }

    pub fn has(&self, id: &String) -> bool {
        self.set.get(id) != None
    }

    // TODO: replace all this with an enum
}

// Required for use in askama templates
impl<'a> IntoIterator for &'a HivePermissionSet {
    type Item = (&'a String, &'a Option<String>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Option<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.iter()
    }
}

impl From<Vec<HivePermission>> for HivePermissionSet {
    fn from(vec: Vec<HivePermission>) -> Self {
        let mut set = HashMap::new();
        for permission in vec {
            set.insert(
                permission.id,
                // Filter out empty scopes generated from nulls. A little hacky, but it works.
                if let Some(scope) = permission.scope {
                    if scope.is_empty() { None } else { Some(scope) }
                } else {
                    None
                },
            );
        }
        Self { set }
    }
}
