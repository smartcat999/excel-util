pub mod command;

extern crate clap;
use clap::App;

#[macro_use]
extern crate lazy_static;

fn main() {
    let mut args = App::new("Excel工具")
        .version("v1.0")
        .author("smartcat")
        .subcommands(vec![
            command::stu::new_sub_command(),
            command::cve::new_sub_command(),
        ])
        .override_usage("etool <command>\n  ");
    let matches = args.clone().get_matches();
    match matches.subcommand() {
        Some(("student", matches)) => {
            command::stu::handler(matches);
        }
        Some(("cve", matches)) => {
            command::cve::handler(matches);
        }
        _ => {
            match args.print_help() {
                Ok(ret) => ret,
                Err(err) => {
                    println!("{:#?}", err);
                }
            };
        }
    };
}
