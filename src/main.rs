pub mod command;

extern crate clap;
use clap::App;

#[macro_use]
extern crate lazy_static;

fn main() {
    let stu_command =command::stu::new_sub_command();
    let mut cve_command = command::cve::new_sub_command();
    let image_command =  command::image::new_sub_command();
    let mut args = App::new("Excel工具")
        .version("v1.0")
        .author("smartcat")
        .subcommands(vec![
            stu_command,
            cve_command.clone(),
            image_command,
        ])
        .override_usage("etool <command>\n  ");
    let matches = args.clone().get_matches();
    match matches.subcommand() {
        Some(("student", matches)) => {
            command::stu::handler(matches);
        }
        Some(("cve", matches)) => {
            match matches.subcommand() {
                Some(("analyze", matches)) => {
                    command::cve::analyze::handler(matches);
                }
                Some(("export", matches)) => {
                    command::cve::exporter::handler(matches);
                }
                _ => cve_command.print_help().unwrap_or_else(|err| {
                    println!("{:#?}", err);
                })
            }
        }
        Some(("image", matches)) => {
            command::image::handler(matches);
        }
        _ => {
            args.print_help().unwrap_or_else(|err| {
                println!("{:#?}", err);
            });
        }
    };
}
