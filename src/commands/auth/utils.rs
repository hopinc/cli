use std::io::Write;

use tabwriter::TabWriter;

pub fn format_users(users: &Vec<&String>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "ID").unwrap();
    }

    for user in users {
        writeln!(&mut tw, "{user}").unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
