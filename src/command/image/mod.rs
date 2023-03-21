use std::{path::Path, fs};

use clap::{App, Arg, ArgMatches, Command};
use crate::command::lib::image;

pub fn new_sub_command<'help>() -> App<'help> {
    Command::new("image")
        .about("整理镜像信息")
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
        .override_usage("etool cve -p ./tmp -o image.json\n  ")
}

pub fn handler(matches: &ArgMatches) {
    let path = matches.get_one::<String>("path").unwrap();
    let _output = matches.get_one::<String>("output").unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    let cmd = image::InspectCmd::new("dockerhub.kubekey.local/huawei/ks-apiserver:v3.3.1-HW");
    image::run_cmd(Box::new(cmd));
}