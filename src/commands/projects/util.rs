use super::types::Project;
use console::style;

pub fn format_projects(projects: &Vec<Project>, default: &Option<String>) -> Vec<String> {
    projects
        .iter()
        .map(|p| {
            format!(
                " {} /{} ({}){}",
                p.name,
                p.namespace,
                p.id,
                if p.id == default.clone().unwrap_or("no_default".to_string()) {
                    style(" (default)").cyan().to_string()
                } else {
                    "".to_string()
                }
            )
        })
        .collect::<Vec<_>>()
}
