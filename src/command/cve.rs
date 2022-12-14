use std::{collections::HashMap, fs, path::Path};

use clap::{App, Arg, ArgMatches, Command};
use office::{DataType, Excel};
use xlsxwriter::{FormatAlignment, FormatColor, Workbook};

pub fn new_sub_command<'help>() -> App<'help> {
    Command::new("cve")
        .about("整理CVE漏洞信息")
        .arg(
            Arg::new("path")
                .default_value("./tmp")
                .short('p')
                .help("生成的目标目录"),
        )
        .arg(
            Arg::new("file")
                .default_value("./Open_Source_Binary_Result.xlsx")
                .short('f')
                .help("待处理的Excel文件路径"),
        )
        .arg(
            Arg::new("sheet")
                .default_value("组件报告")
                .long("sheet")
                .help("待处理的Excel文件表格名称"),
        )
        .override_usage("etool cve -p ./tmp -f Open_Source_Binary_Result.xlsx --sheet 组件报告\n  ")
}

pub fn handler(matches: &ArgMatches) {
    // println!("{:#?}", matches)
    let path = matches.get_one::<String>("path").unwrap();
    let file = matches.get_one::<String>("file").unwrap();
    let sheet = matches.get_one::<String>("sheet").unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    let mut workbook = Excel::open(file).unwrap();
    let range = workbook.worksheet_range(sheet).unwrap();
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut object_index: usize = 0;
    let mut vulnerability_index: usize = 0;
    let mut object_map: HashMap<String, Vec<String>> = HashMap::new();
    for (index, vals) in range.rows().enumerate() {
        if index == 0 {
            println!("{:#?}", vals);
            for (header_idx, header_content) in vals.iter().enumerate() {
                if let office::DataType::String(val) = header_content {
                    // println!("{:#?}", val);
                    if val == "Component" {
                        component_index = header_idx;
                    } else if val == "Version" {
                        version_index = header_idx;
                    } else if val == "Object full path" {
                        object_index = header_idx;
                    } else if val == "Vulnerability count" {
                        vulnerability_index = header_idx;
                    }
                }
            }
            println!(
                "object_index:{:#?}\ncomponent_index:{:#?}\nversion_index:{:#?}\nvulnerability_index:{:#?}",
                object_index, component_index, version_index, vulnerability_index
            );
            if component_index == version_index
                || component_index == object_index
                || component_index == vulnerability_index
                || version_index == object_index
                || version_index == vulnerability_index
                || object_index == vulnerability_index
            {
                break;
            }
        } else {
            let vulnerability = match vals.get(vulnerability_index) {
                Some(v) => match v {
                    DataType::String(v) => v,
                    DataType::Int(_)
                    | DataType::Float(_)
                    | DataType::Bool(_)
                    | DataType::Error(_)
                    | DataType::Empty => "0",
                },
                None => "0",
            };

            let vulnerability = vulnerability.parse::<usize>().unwrap();
            if vulnerability == 0 {
                continue;
            }

            let component = match vals.get(component_index) {
                Some(v) => match v {
                    DataType::String(v) => v,
                    DataType::Int(_)
                    | DataType::Float(_)
                    | DataType::Bool(_)
                    | DataType::Error(_)
                    | DataType::Empty => "",
                },
                None => "",
            };

            let version = match vals.get(version_index) {
                Some(v) => match v {
                    DataType::String(v) => v,
                    DataType::Int(_)
                    | DataType::Float(_)
                    | DataType::Bool(_)
                    | DataType::Error(_)
                    | DataType::Empty => "",
                },
                None => "",
            };
            let object = match vals.get(object_index) {
                Some(v) => {
                    let v = match v {
                        DataType::String(v) => {
                            let mut object_key: String = String::new();
                            let secs: Vec<&str> = v.split('/').collect();
                            for &sec in secs.iter() {
                                if sec.contains("dockerhub.kubekey.local#") {
                                    object_key = sec.to_string();
                                }
                            }
                            object_key
                        }
                        DataType::Int(_)
                        | DataType::Float(_)
                        | DataType::Bool(_)
                        | DataType::Error(_)
                        | DataType::Empty => String::from(""),
                    };
                    v
                }
                None => String::from(""),
            };
            let mut object: Vec<&str> = object.split('#').collect();
            if let Some(object) = object.pop() {
                let object = object.trim_end_matches(".tar_");
                println!(
                    "object:{:#?}\ncomponent:{:#?}\nversion:{:#?}\nvulnerability:{}",
                    object, component, version, vulnerability
                );
                if object.is_empty() || component.is_empty() || version.is_empty() {
                    continue;
                }
                let component = format!("{}{}:  {}", component, version, vulnerability);
                if let Some(components) = object_map.get_mut(object) {
                    if !components.contains(&component) {
                        components.push(component);
                    }
                } else {
                    object_map.insert(object.to_string(), vec![component]);
                }
            }
        }
    }

    println!("{:#?}", object_map.len());
    if !object_map.is_empty() {
        let output = format!("{}/{}", path, "cve.xlsx");
        println!("{:#?}", output);
        let out = Workbook::new(output.as_str());
        let format1 = out
            .add_format()
            .set_align(FormatAlignment::Center)
            .set_bg_color(FormatColor::Red);
        let format2 = out.add_format().set_align(FormatAlignment::Center);
        let mut sheet1 = out.add_worksheet(None).unwrap();
        sheet1.write_string(0, 0, "image", Some(&format1)).unwrap();
        sheet1
            .write_string(0, 1, "dependencies", Some(&format1))
            .unwrap();

        for (index, (k, v)) in object_map.iter().enumerate() {
            println!("{:#?}: \n{:#?}", k, v);
            sheet1
                .write_string((index + 1) as u32, 0, k, Some(&format2))
                .unwrap();
            sheet1
                .write_string((index + 1) as u32, 1, v.join("\n").as_str(), Some(&format2))
                .unwrap();
        }
    }
}
