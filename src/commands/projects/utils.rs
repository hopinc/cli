use std::io::Write;

use anyhow::{bail, Result};
use regex::Regex;
use tabwriter::TabWriter;

use crate::state::http::HttpClient;

use super::types::{CreateProject, Project, SingleProjectResponse};

pub fn format_projects(projects: &Vec<Project>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "NAME\tNAMESPACE\tID\tCREATED\tTYPE").unwrap();
    }

    for project in projects {
        writeln!(
            &mut tw,
            "{}\t/{}\t{}\t{}\t{}",
            project.name.clone(),
            project.namespace,
            project.id,
            project.created_at,
            project.type_,
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn format_project(project: &Project) -> String {
    format_projects(&vec![project.clone()], false)[0].clone()
}

pub async fn create_project(
    http: &HttpClient,
    name: &str,
    namespace: &str,
    payment_method_id: &str,
) -> Result<Project> {
    let data = http
        .request::<SingleProjectResponse>(
            "POST",
            "/projects",
            Some((
                serde_json::to_vec(&CreateProject {
                    name: name.to_string(),
                    namespace: namespace.to_string(),
                    payment_method_id: payment_method_id.to_string(),
                })
                .unwrap()
                .into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?
        .project;

    Ok(data)
}

pub fn validate_namespace(namespace: &str) -> Result<()> {
    let regex = Regex::new(r"(?i)[a-z0-9_]")?;

    if namespace.len() > 15 {
        bail!("Namespace must be less than 15 characters")
    } else if !regex.is_match(namespace) {
        bail!("Namespace must contain only letters, numbers and underscores")
    }
    
    Ok(())
}
