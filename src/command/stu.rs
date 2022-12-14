use std::{fs, path::Path};

use clap::{App, Arg, ArgMatches, Command};
use office::{DataType, Excel};

pub fn new_sub_command<'help>() -> App<'help> {
    Command::new("student")
        .about("处理学生表格信息")
        .arg(
            Arg::new("path")
                .default_value("./tmp")
                .short('p')
                .help("生成的目标目录"),
        )
        .arg(
            Arg::new("file")
                .default_value("./student.xlsx")
                .short('f')
                .help("待处理的Excel文件路径"),
        )
        .arg(
            Arg::new("sheet")
                .default_value("Sheet1")
                .long("sheet")
                .help("待处理的Excel文件表格名称"),
        )
        .arg(
            Arg::new("column")
                .default_value("3")
                .long("col")
                .help("待处理的表格列序号(从0开始)"),
        )
        .arg(
            Arg::new("row")
                .default_value("2")
                .long("row")
                .help("待处理的表格行序号(从0开始)"),
        )
        .override_usage("etool student -p ./tmp -f student.xlsx --sheet Sheet1 --row 2 --col 3\n  ")
}

pub fn handler(matches: &ArgMatches) {
    // println!("{:#?}", matches);
    let path = matches.get_one::<String>("path").unwrap();
    let file = matches.get_one::<String>("file").unwrap();
    let sheet = matches.get_one::<String>("sheet").unwrap();
    let row = matches.get_one::<String>("row").unwrap();
    let column = matches.get_one::<String>("column").unwrap();
    println!("{}行-{}列", row, column);
    let row = row.parse::<usize>().unwrap();
    let column = column.parse::<usize>().unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };
    let mut workbook = Excel::open(file).unwrap();
    let range = workbook.worksheet_range(sheet).unwrap();
    for (index, vals) in range.rows().enumerate() {
        if index < row {
            continue;
        }

        let value = vals[column].clone();
        if let DataType::String(v) = value {
            let sub_path = format!("{}/{}", path, v);
            println!("{:#?}", sub_path);
            if !Path::exists(Path::new(&sub_path)) {
                fs::create_dir(sub_path).unwrap();
            }
        }
    }
}
