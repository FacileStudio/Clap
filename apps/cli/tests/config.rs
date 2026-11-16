//! The config file editor, and what a person may type for a server or a project.

use clap_cli::config::{flat, site};
use clap_cli::url::project_reference;

#[test]
fn the_config_file_is_edited_line_by_line() {
    let text = "# my settings\nserver_url: \"https://clap.example/api\"\ntheme: dark # kept\n";
    assert_eq!(flat::get(text, "server_url").as_deref(), Some("https://clap.example/api"));
    assert_eq!(flat::get(text, "theme").as_deref(), Some("dark"));
    assert_eq!(flat::get(text, "token"), None);
    let logged_in = flat::set(&flat::set(text, "server_url", "https://clap.example"), "token", "s3cret");
    assert_eq!(logged_in, "# my settings\nserver_url: https://clap.example\ntheme: dark # kept\ntoken: s3cret\n");
    assert_eq!(
        flat::remove(&logged_in, "token"),
        "# my settings\nserver_url: https://clap.example\ntheme: dark # kept\n"
    );
    assert_eq!(flat::remove("token: x\n", "token"), "");
}

#[test]
fn servers_and_project_references_are_normalized() {
    for typed in
        ["https://clap.example", "https://clap.example/", "https://clap.example/api", " https://clap.example/api/ "]
    {
        assert_eq!(site(typed), "https://clap.example", "{typed}");
    }
    assert_eq!(project_reference("d1"), (None, "d1".to_owned()));
    let from_url = project_reference("https://clap.example/projects/3f2a?present=1");
    assert_eq!(from_url, (Some("https://clap.example".to_owned()), "3f2a".to_owned()));
}
