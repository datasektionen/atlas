use std::collections::HashSet;

use serde::{Deserialize, Serialize, de, ser::SerializeMap};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct HivePermissionSet {
    set: HashSet<HivePermission>,
}

impl From<HashSet<HivePermission>> for HivePermissionSet {
    fn from(set: HashSet<HivePermission>) -> Self {
        Self { set }
    }
}

impl HivePermissionSet {
    pub fn has(&self, perm: &HivePermission) -> bool {
        self.set.contains(perm)
    }
}

// Serde implementation for this is manually written further down due to the derived implementation
// not being able to handle unit variants when scope is provided anyway. If anyone knows how to
// tell serde to ignore such data, please do that here and remove all excess code!
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
// #[serde(key = "id", content = "scope", rename_all = "kebab-case")]
pub enum HivePermission {
    // New permissions need to be added in Self::{key, scope, create} due to the above mentioned
    // serde bullshit.
    Post,
    ManageTags,
    UseTag(TagScope),

    // Matches all unknown permissions, can be ignored
    Unknown,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
#[serde(from = "String")]
pub enum TagScope {
    Wildcard,
    Tag(String),
}

impl From<String> for TagScope {
    fn from(s: String) -> Self {
        if s == "*" {
            Self::Wildcard
        } else {
            Self::Tag(s)
        }
    }
}

impl From<&TagScope> for String {
    fn from(scope: &TagScope) -> Self {
        match scope {
            TagScope::Wildcard => "*".to_string(),
            TagScope::Tag(s) => s.clone(),
        }
    }
}

// Begin serde bullshit implementation
impl HivePermission {
    pub fn key(&self) -> &str {
        match self {
            Self::Post => "post",
            Self::ManageTags => "manage-tags",
            Self::UseTag(_) => "use-tag",

            Self::Unknown => "unknown",
        }
    }

    pub fn scope(&self) -> Option<String> {
        match self {
            Self::UseTag(scope) => Some(scope.into()),
            _ => None,
        }
    }

    // Incredibly ugly
    fn create(id: Option<String>, scope: Option<String>) -> Option<Self> {
        if let Some(id) = id {
            Some(match id.as_str() {
                "post" => Self::Post,
                "manage-tags" => Self::ManageTags,
                "use-tag" => Self::UseTag(scope?.to_string().into()),
                _ => Self::Unknown,
            })
        } else {
            Some(Self::Unknown)
        }
    }
}

impl Serialize for HivePermission {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = s.serialize_map(Some(2))?;
        map.serialize_entry("id", self.key())?;
        map.serialize_entry("scope", &self.scope())?;
        map.end()
    }
}

struct HivePermissionVisitor;

impl<'de> de::Visitor<'de> for HivePermissionVisitor {
    type Value = HivePermission;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map containing 'id' and 'scope'")
    }

    fn visit_map<M>(self, mut access: M) -> Result<HivePermission, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let mut id: Option<String> = None;
        let mut scope: Option<String> = None;
        while let Some((key, value)) = access.next_entry::<String, Option<String>>()? {
            if key == "id" {
                id = value;
            } else if key == "scope" {
                scope = value;
            }
        }
        match HivePermission::create(id, scope) {
            Some(perm) => Ok(perm),
            // This is the only possible error
            None => Err(de::Error::missing_field("scope")),
        }
    }
}

impl<'de> Deserialize<'de> for HivePermission {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_map(HivePermissionVisitor)
    }
}
// End serde bullshit implementation
