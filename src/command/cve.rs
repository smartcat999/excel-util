use std::{collections::HashMap, fs, path::Path};

use clap::{value_parser, App, Arg, ArgAction, ArgMatches, Command};
use office::{DataType, Excel};
use xlsxwriter::{FormatAlignment, FormatColor, Workbook};

const COLUMN_FILED_COMPONENT: &str = "Component";
const COLUMN_FILED_VERSION: &str = "Version";
const COLUMN_FILED_OBJECT: &str = "Object full path";
const COLUMN_FILED_VUL_COUNT: &str = "Vulnerability count";
const COLUMN_FILED_CVE: &str = "CVE";

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
                .action(ArgAction::Append)
                .value_parser(value_parser!(String))
                .default_values(&["./Open_Source_Binary_Result.xlsx"])
                .short('f')
                .help("待处理的Excel文件路径"),
        )
        .arg(
            Arg::new("sheet")
                .default_value("组件报告")
                .long("sheet")
                .help("待处理的Excel文件表格名称"),
        )
        .arg(
            Arg::new("sheet_ext")
                .default_value("漏洞报告")
                .long("sheet_ext")
                .help("待处理的Excel文件表格名称"),
        )
        .arg(
            Arg::new("output")
                .default_value("cve.xlsx")
                .short('o')
                .help("输出的Excel文件表格名称"),
        )
        .override_usage("etool cve -p ./tmp -f Open_Source_Binary_Result.xlsx --sheet 组件报告 --sheet_ext 漏洞报告 -o cve.xlsx\n  ")
}

pub fn handler(matches: &ArgMatches) {
    let path = matches.get_one::<String>("path").unwrap();
    let files: Vec<&String> = matches
        .get_many::<String>("file")
        .unwrap()
        .collect::<Vec<&String>>();
    let sheet = matches.get_one::<String>("sheet").unwrap();
    let sheet_ext = matches.get_one::<String>("sheet_ext").unwrap();
    let output = matches.get_one::<String>("output").unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    // output
    let output = format!("{}/{}", path, output);
    let mut out = Workbook::new(output.as_str());

    let mut object_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut component_map: HashMap<String, Vec<String>> = HashMap::new();

    for file in files.iter() {
        let mut workbook = Excel::open(file).unwrap();

        // parse component's cve
        parse_component_detail(&mut workbook, sheet_ext, &mut component_map);

        // parse object's component
        parse_object(&mut workbook, sheet, &mut object_map);
    }

    write_component_output(&component_map, &mut out);
    write_object_output(&object_map, &component_map, &mut out);
    println!(
        "object num: {:?}\ncomponent num: {:?}",
        object_map.len(),
        component_map.len()
    );
    println!("inuput: {:#?}\noutput: {:#?}", files, output);
}

fn write_object_output(
    object_map: &HashMap<String, Vec<String>>,
    component_map: &HashMap<String, Vec<String>>,
    out: &mut Workbook,
) {
    if !object_map.is_empty() {
        let format1 = out
            .add_format()
            .set_align(FormatAlignment::Center)
            .set_bg_color(FormatColor::Red);
        let format2 = out.add_format().set_align(FormatAlignment::Left);
        let mut sheet1 = out.add_worksheet(Some("image")).unwrap();
        sheet1.write_string(0, 0, "image", Some(&format1)).unwrap();
        sheet1
            .write_string(0, 1, "dependencies", Some(&format1))
            .unwrap();
        sheet1.write_string(0, 2, "cves", Some(&format1)).unwrap();
        sheet1.write_string(0, 3, "num", Some(&format1)).unwrap();

        let mut object_keys: Vec<String> = object_map.keys().map(|x| x.to_string()).collect();
        object_keys.sort();
        // println!("object: {:#?}", object_keys);
        for (index, k) in object_keys.iter().enumerate() {
            // println!("{:#?}: \n{:#?}", k, v);
            if let Some(v) = object_map.get(k) {
                sheet1
                    .write_string((index + 1) as u32, 0, k, Some(&format2))
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 1, v.join("\n").as_str(), Some(&format2))
                    .unwrap();

                // add row of cves
                let mut cves: Vec<String> = Vec::new();
                for comp_key in v.iter() {
                    let comp_key: Vec<&str> = comp_key.split(":  ").collect();
                    let comp_key = comp_key[0];
                    if let Some(cve) = component_map.get(comp_key) {
                        cves.extend_from_slice(cve);
                    }
                }
                sheet1
                    .write_string(
                        (index + 1) as u32,
                        2,
                        cves.join("\n").as_str(),
                        Some(&format2),
                    )
                    .unwrap();
                sheet1
                    .write_number((index + 1) as u32, 3, cves.len() as f64, Some(&format2))
                    .unwrap();
            }
        }
    }
}

fn parse_object(workbook: &mut Excel, sheet: &str, object_map: &mut HashMap<String, Vec<String>>) {
    let range = workbook.worksheet_range(sheet).unwrap();
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut object_index: usize = 0;
    let mut vulnerability_index: usize = 0;

    for (index, vals) in range.rows().enumerate() {
        if index == 0 {
            for (header_idx, header_content) in vals.iter().enumerate() {
                if let office::DataType::String(val) = header_content {
                    if val == COLUMN_FILED_COMPONENT {
                        component_index = header_idx;
                    } else if val == COLUMN_FILED_VERSION {
                        version_index = header_idx;
                    } else if val == COLUMN_FILED_OBJECT {
                        object_index = header_idx;
                    } else if val == COLUMN_FILED_VUL_COUNT {
                        vulnerability_index = header_idx;
                    }
                }
            }
            // println!(
            //     "object_index:{:#?} component_index:{:#?} version_index:{:#?} vulnerability_index:{:#?}",
            //     object_index, component_index, version_index, vulnerability_index
            // );
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
                // println!(
                //     "object:{:#?}\ncomponent:{:#?}\nversion:{:#?}\nvulnerability:{}",
                //     object, component, version, vulnerability
                // );
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
}

fn write_component_output(component_map: &HashMap<String, Vec<String>>, out: &mut Workbook) {
    if !component_map.is_empty() {
        let format1 = out
            .add_format()
            .set_align(FormatAlignment::Center)
            .set_bg_color(FormatColor::Red);
        let format2 = out.add_format().set_align(FormatAlignment::Left);
        let mut sheet1 = out.add_worksheet(Some("cve")).unwrap();
        sheet1
            .write_string(0, 0, "component", Some(&format1))
            .unwrap();
        sheet1.write_string(0, 1, "cve", Some(&format1)).unwrap();
        sheet1.write_string(0, 2, "num", Some(&format1)).unwrap();

        let mut component_keys: Vec<String> = component_map.keys().map(|x| x.to_string()).collect();
        component_keys.sort();

        for (index, k) in component_keys.iter().enumerate() {
            if let Some(v) = component_map.get(k) {
                sheet1
                    .write_string((index + 1) as u32, 0, k, Some(&format2))
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 1, v.join("\n").as_str(), Some(&format2))
                    .unwrap();
                sheet1
                    .write_number((index + 1) as u32, 2, v.len() as f64, Some(&format2))
                    .unwrap();
            }
        }
    }
}

fn parse_component_detail(
    workbook: &mut Excel,
    sheet: &str,
    component_map: &mut HashMap<String, Vec<String>>,
) {
    let range = workbook.worksheet_range(sheet).unwrap();
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut cve_index: usize = 0;

    for (index, vals) in range.rows().enumerate() {
        if index == 0 {
            for (header_idx, header_content) in vals.iter().enumerate() {
                if let office::DataType::String(val) = header_content {
                    if val == COLUMN_FILED_COMPONENT {
                        component_index = header_idx;
                    } else if val == COLUMN_FILED_VERSION {
                        version_index = header_idx;
                    } else if val == COLUMN_FILED_CVE {
                        cve_index = header_idx;
                    }
                }
            }
            // println!(
            //     "component_index:{:#?} version_index:{:#?} cve_index:{:#?}",
            //     component_index, version_index, cve_index
            // );
            if component_index == version_index
                || component_index == cve_index
                || version_index == cve_index
            {
                break;
            }
        } else {
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
            let cve = match vals.get(cve_index) {
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

            if cve.is_empty() || component.is_empty() || version.is_empty() {
                continue;
            }
            let component = format!("{}{}", component, version);
            let cve_inner = cve.to_string();
            if let Some(components) = component_map.get_mut(&component) {
                if !components.contains(&cve_inner) {
                    components.push(cve_inner);
                }
            } else {
                component_map.insert(component.to_string(), vec![cve_inner]);
            }
        }
    }
}
