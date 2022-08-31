use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Build {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleBuild {
    pub build: Build,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "e", content = "d")]
pub enum BuildEvents {
    #[serde(rename = "BUILD_PROGRESS")]
    BuildProgress(BuildProgress),
    #[serde(rename = "BUILD_CANCELLED")]
    BuildCancelled(BuildEvent),
    #[serde(rename = "PUSH_SUCCESS")]
    PushSuccess(BuildEvent),
    #[serde(rename = "PUSH_FAILURE")]
    PushFailure(BuildEvent),
}

#[derive(Debug, Deserialize)]
pub struct BuildProgress {
    pub build_id: String,
    pub deployment_id: String,
    pub id: String,
    pub log: String,
    pub sent_at: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildEvent {
    pub build_id: String,
    pub deployment_id: String,
}
