use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub model_dir: String,
    pub profile: String,
    pub license_accepted: bool,
    pub checked_profile: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub kind: String,
    pub phase: String,
    pub detail: String,
    pub completed: Option<u64>,
    pub total: Option<u64>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub id: String,
    pub path: String,
    pub prompt: String,
    pub effective_prompt: String,
    pub mode: String,
    pub width: u32,
    pub height: u32,
    pub seed: u32,
    pub has_alpha: bool,
    #[serde(default)]
    pub transparency_requested: bool,
    pub profile: String,
    pub created_at: u64,
    pub references: Vec<String>,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub provenance: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: String,
    pub path: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    pub prompt: String,
    pub mode: String,
    pub width: u32,
    pub height: u32,
    pub seed: Option<u32>,
    pub transparent: bool,
    pub references: Vec<String>,
    pub parent_id: Option<String>,
}

impl GenerateRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() || self.prompt.len() > 96_000 {
            return Err("Enter an image prompt of up to 24,000 characters.".into());
        }
        if !["create", "transform", "edit"].contains(&self.mode.as_str()) {
            return Err("Unknown studio mode.".into());
        }
        if self.references.len() > 10 || (self.mode != "create" && self.references.is_empty()) {
            return Err("Add between one and ten reference images for Transform or Edit.".into());
        }
        if !(256..=4096).contains(&self.width)
            || !(256..=4096).contains(&self.height)
            || !self.width.is_multiple_of(16)
            || !self.height.is_multiple_of(16)
            || u64::from(self.width) * u64::from(self.height) > 4_400_000
        {
            return Err(
                "Use image dimensions in multiples of 16 within the supported size.".into(),
            );
        }
        if self.seed.is_some_and(|seed| seed > 2_147_483_647) {
            return Err("Seed must be between 0 and 2147483647.".into());
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub gpu_name: Option<String>,
    pub vram_mb: Option<u64>,
    pub ram_bytes: u64,
    pub free_bytes: u64,
    pub model_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub preferences: Preferences,
    pub hardware: Hardware,
    pub model_downloaded: bool,
    pub runtime_installed: bool,
    pub ready: bool,
    pub operation: Option<Operation>,
    pub artworks: Vec<Artwork>,
    pub data_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> GenerateRequest {
        GenerateRequest {
            prompt: "An orchid".into(),
            mode: "create".into(),
            width: 1024,
            height: 1024,
            seed: None,
            transparent: false,
            references: vec![],
            parent_id: None,
        }
    }
    #[test]
    fn rejects_unbounded_dimensions_and_invalid_seed() {
        let mut input = request();
        input.width = 4096;
        input.height = 4096;
        assert!(input.validate().is_err());
        input.width = 1024;
        input.height = 1024;
        input.seed = Some(u32::MAX);
        assert!(input.validate().is_err());
    }
    #[test]
    fn image_modes_need_bounded_references() {
        let mut input = request();
        input.mode = "edit".into();
        assert!(input.validate().is_err());
        input.references.push("owned.png".into());
        assert!(input.validate().is_ok());
        input.references = vec!["owned.png".into(); 11];
        assert!(input.validate().is_err());
    }
}
