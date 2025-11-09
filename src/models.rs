use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SecretGetRequest {
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Serialize)]
pub struct SecretsGetRequest {
    #[serde(rename = "IDS")]
    pub ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct ListItem {
    pub id: String,
    #[allow(dead_code)]
    #[serde(rename = "organizationId")]
    pub organization_id: Option<String>,
    pub key: String,
}

#[derive(Deserialize)]
pub struct ListResponse {
    pub data: Vec<ListItem>,
}

#[derive(Serialize)]
pub struct SecretsDeleteRequest {
    #[serde(rename = "IDS")]
    pub ids: Vec<String>,
}

#[derive(Serialize)]
pub struct SecretCreateRequest {
    pub key: String,
    pub value: String,
    pub note: Option<String>,
    #[serde(rename = "OrganizationID")]
    pub organization_id: Option<String>,
    #[serde(rename = "ProjectIDS")]
    pub project_ids: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct SecretPutRequest {
    pub id: String,
    pub key: String,
    pub value: String,
    pub note: Option<String>,
    #[serde(rename = "OrganizationID")]
    pub organization_id: Option<String>,
    #[serde(rename = "ProjectIDS")]
    pub project_ids: Option<Vec<String>>,
}
