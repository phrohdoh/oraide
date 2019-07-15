use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TraitProperty {
    pub kind: TraitPropertyKind,
    pub type_name: String,
    pub human_friendly_type_name: String,
    pub name: String,

    pub doc_lines: Option<Vec<String>>,
    pub default_value: Option<String>,
    pub valid_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitPropertyKind {
    Single,
    Multi,
    Choice,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NamespacedType {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TraitDetail {
    pub defining_assembly_name: String,
    pub is_conditional: bool,
    pub required_traits: Vec<NamespacedType>,
    pub properties: Vec<TraitProperty>,

    pub doc_lines: Option<Vec<String>>,

    // `TraitDetail` is also a `NamespacedType`
    pub namespace: String,
    pub name: String,
}