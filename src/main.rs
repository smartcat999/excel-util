extern crate clap;
use clap::{App, Arg};
use office::{DataType, Excel, Range};
use std::fs;
use std::path::Path;
fn main() {
    let args = App::new("Excel工具")
        .version("v1.0")
        .author("smartcat")
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
                .help("待处理的表格行号"),
        )
        .override_usage(
            "etool -p ./tmp -f student.xlsx --sheet Sheet1 --col 3\n  "
        )
        .get_matches();
    let path = args.get_one::<String>("path").unwrap();
    let file = args.get_one::<String>("file").unwrap();
    let sheet = args.get_one::<String>("sheet").unwrap();
    let column = args.get_one::<String>("column").unwrap();
    print!("{}", column);
    let column = column.parse::<usize>().unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path);
    };
    let mut workbook = Excel::open(file).unwrap();
    let range = workbook.worksheet_range(sheet).unwrap();
    for i in range.rows() {
        let value = i[column].clone();
        match value {
            DataType::String(v) => {
                let sub_path = format!("{}/{}", path, v);
                println!("{:#?}", sub_path);
                if !Path::exists(Path::new(&sub_path)) {
                    fs::create_dir(sub_path).unwrap();
                }
            }
            _ => {}
        }
    }
}
