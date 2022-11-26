use std::io::Write;

use anyhow::Result;
use tabwriter::TabWriter;

use crate::state::http::HttpClient;

use super::types::{CreateParams, Project, SingleProjectResponse};

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

pub async fn create_project(http: &HttpClient, name: &str, namespace: &str) -> Result<Project> {
    let json = http
        .request::<SingleProjectResponse>(
            "POST",
            "/projects",
            Some((
                serde_json::to_vec(&CreateParams {
                    name: name.to_string(),
                    namespace: namespace.to_string(),
                })
                .unwrap()
                .into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?;

    Ok(json.project)
}
