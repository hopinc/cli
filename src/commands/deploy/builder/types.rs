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
#[serde(tag = "e", content = "d", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BuildEvents {
    BuildProgress(BuildProgress),
    BuildCancelled(BuildEvent),
    PushSuccess(BuildEvent),
    PushFailure(BuildEvent),
    BuildUpdate(BuildValidationWrapper),
    BuildCreate(BuildValidationWrapper),
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

#[derive(Debug, Deserialize)]
pub struct BuildValidationWrapper {
    pub build: BuildValidationEvent,
}

#[derive(Debug, Deserialize)]
pub struct BuildValidationEvent {
    pub deployment_id: String,
    pub id: String,
    pub state: BuildStatus,
    pub validation_failure: Option<ValidationFailure>,
}

#[derive(Debug, Deserialize)]
pub struct ValidationFailure {
    pub reason: String,
    pub help_link: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    Pending,
    Validating,
    ValidationFailed,
}
