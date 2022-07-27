use std::io::Write;

use console::style;
use tabwriter::TabWriter;

use super::types::Project;

pub fn format_projects(
    projects: &Vec<Project>,
    default: &Option<String>,
    title: bool,
) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(
            &mut tw,
            "{}",
            style("NAME\tNAMESPACE\tID\tCREATED\tTYPE").white()
        )
        .unwrap();
    }

    for project in projects {
        let data = format!(
            "{}\t{}\t{}\t{}\t{}",
            project.name.clone(),
            project.namespace,
            project.id,
            project.created_at,
            project.p_type
        );

        // because the tabwriter lib counts characters we need to style all output
        let content = match default {
            Some(default) => {
                if &project.id == default {
                    style(data).green().to_string()
                } else {
                    style(data).white().to_string()
                }
            }
            None => style(data).white().to_string(),
        };

        writeln!(&mut tw, "{}", content).unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect()
}
