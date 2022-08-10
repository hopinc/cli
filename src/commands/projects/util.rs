use std::io::Write;

use tabwriter::TabWriter;

use super::types::Project;

pub fn format_projects(projects: &Vec<Project>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "NAME\tNAMESPACE\tID\tCREATED\tTYPE").unwrap();
    }

    for project in projects {
        writeln!(
            &mut tw,
            "{}\t{}\t{}\t{}\t{}",
            project.name.clone(),
            project.namespace,
            project.id,
            project.created_at,
            project.p_type,
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
