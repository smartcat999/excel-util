use std::{fs, path::Path};

use crate::command::lib::image;
use clap::{value_parser, App, Arg, ArgAction, ArgMatches, Command};

pub fn new_sub_command<'help>() -> App<'help> {
    Command::new("image")
        .about("整理镜像信息")
        .arg(
            Arg::new("file")
                .action(ArgAction::Append)
                .value_parser(value_parser!(String))
                .default_values(&["./image.txt"])
                .short('f')
                .help("待处理的镜像列表文件路径"),
        )
        .arg(
            Arg::new("path")
                .default_value("./tmp")
                .short('p')
                .help("生成的目标目录"),
        )
        .arg(
            Arg::new("output")
                .default_value("image.json")
                .short('o')
                .help("输出的镜像信息文件名称"),
        )
        .override_usage("etool cve -f ./image.txt -p ./tmp -o image.json\n  ")
}

pub fn handler(matches: &ArgMatches) {
    let files: Vec<&String> = matches
        .get_many::<String>("file")
        .unwrap()
        .collect::<Vec<&String>>();
    let path = matches.get_one::<String>("path").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    image::dump(files, path, output);
}
