use std::{collections::HashMap, fs, path::Path, sync::Mutex};

use clap::{value_parser, App, Arg, ArgAction, ArgMatches, Command};
use office::{DataType, Excel};
use xlsxwriter::{Format, FormatAlignment, FormatColor, Workbook};

pub mod api;
pub mod lib;

use lib::CveApis;

const COLUMN_FILED_COMPONENT: &str = "Component";
const COLUMN_FILED_VERSION: &str = "Version";
const COLUMN_FILED_OBJECT: &str = "Object full path";
const COLUMN_FILED_VUL_COUNT: &str = "Vulnerability count";
const COLUMN_FILED_CVE: &str = "CVE";
const COLUMN_FILED_BINARY_OBJECT: &str = "Object";

const TITLE_FONT_SIZE: f64 = 16.0;

lazy_static! {
    pub static ref CVE_API: Mutex<CveApis> = {
        let mut cve_apis = lib::CveApis::new();
        cve_apis.register(Box::new(api::aliyun_api::AliyunApi::new()));
        Mutex::new(cve_apis)
    };
}

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
        .arg(
            Arg::new("detail")
                .long("detail")
                .action(clap::ArgAction::SetTrue)
                .help("是否输出CVE详细信息"),
        )
        .override_usage("etool cve -p ./tmp -f Open_Source_Binary_Result.xlsx --sheet 组件报告 --sheet_ext 漏洞报告 --detail -o cve.xlsx\n  ")
}

pub fn handler(matches: &ArgMatches) {
    let path = matches.get_one::<String>("path").unwrap();
    let files: Vec<&String> = matches
        .get_many::<String>("file")
        .unwrap()
        .collect::<Vec<&String>>();
    let sheet = matches.get_one::<String>("sheet").unwrap();
    let sheet_ext = matches.get_one::<String>("sheet_ext").unwrap();
    let detail = matches.get_flag("detail");
    let output = matches.get_one::<String>("output").unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    // output
    let output = format!("{}/{}", path, output);
    let mut out = Workbook::new(output.as_str());

    let mut object_map: HashMap<String, HashMap<String, Vec<CveComponent>>> = HashMap::new();
    let mut component_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut cve_map: HashMap<String, String> = HashMap::new();

    for file in files.iter() {
        let mut workbook = Excel::open(file).unwrap();

        // parse component's cve
        parse_component_cves(&mut workbook, sheet_ext, &mut component_map, &mut cve_map);

        // parse object's component
        parse_object(&mut workbook, sheet, &mut object_map);
    }

    write_component_output(&component_map, &mut out);
    write_object_output(&object_map, &component_map, &mut out);
    write_cve_output(&cve_map, &mut out, detail);
    println!(
        "object num: {:?}\ncomponent num: {:?}\ncve num: {:?}",
        object_map.len(),
        component_map.len(),
        cve_map.len()
    );
    println!("inuput: {:#?}\noutput: {:#?}", files, output);
}

// format init
fn set_title_format(out: &Workbook) -> Format {
    out.add_format()
        .set_align(FormatAlignment::Center)
        .set_bg_color(FormatColor::Yellow)
        .set_font_size(TITLE_FONT_SIZE)
}

fn set_content_format(out: &Workbook) -> Format {
    out.add_format()
        .set_align(FormatAlignment::Center)
        .set_align(FormatAlignment::VerticalCenter)
}

#[derive(Debug)]
struct CveComponent {
    component: String,
    binary: String,
    cve: usize,
}

impl CveComponent {
    fn new(component: String, binary: String, cve: usize) -> CveComponent {
        CveComponent {
            component,
            binary,
            cve,
        }
    }

    fn id(&self) -> String {
        self.component.clone()
    }
}

fn write_object_output(
    object_map: &HashMap<String, HashMap<String, Vec<CveComponent>>>,
    component_map: &HashMap<String, Vec<String>>,
    out: &mut Workbook,
) {
    if !object_map.is_empty() {
        let format1 = set_title_format(out);
        let format2 = set_content_format(out);
        let mut sheet1 = out.add_worksheet(Some("image")).unwrap();
        sheet1.write_string(0, 0, "image", Some(&format1)).unwrap();

        sheet1
            .write_string(0, 1, "image_cve", Some(&format1))
            .unwrap();
        sheet1
            .write_string(0, 2, "component", Some(&format1))
            .unwrap();
        sheet1
            .write_string(0, 3, "component_cve", Some(&format1))
            .unwrap();
        sheet1.write_string(0, 4, "cve", Some(&format1)).unwrap();

        sheet1.write_string(0, 5, "object", Some(&format1)).unwrap();

        let mut object_keys: Vec<String> = object_map.keys().map(|x| x.to_string()).collect();
        object_keys.sort();

        let mut global_index: usize = 1;
        for (_, k) in object_keys.iter().enumerate() {
            let image_merge_start = global_index as u32;
            let mut image_merge_end = image_merge_start - 1;
            let mut image_cve: usize = 0;
            if let Some(v) = object_map.get(k) {
                for (comp_id, comps) in v.iter() {
                    let comp_merge_start = global_index as u32;
                    let mut comp_merge_end = comp_merge_start - 1;
                    let mut comp_cve = 0;
                    for comp in comps.iter() {
                        sheet1
                            .write_string(global_index as u32, 0, k, Some(&format2))
                            .unwrap();
                        sheet1
                            .write_string(global_index as u32, 2, &comp.id(), Some(&format2))
                            .unwrap();
                        if component_map.contains_key(&comp.id()) {
                            sheet1
                                .write_string(global_index as u32, 5, &comp.binary, Some(&format2))
                                .unwrap();
                            sheet1
                                .write_number(
                                    global_index as u32,
                                    3,
                                    comp.cve as f64,
                                    Some(&format2),
                                )
                                .unwrap();
                            if let Some(comp_cve) = component_map.get(&comp.id()) {
                                sheet1
                                    .write_string(
                                        global_index as u32,
                                        4,
                                        comp_cve.join("\n").as_str(),
                                        Some(&format2),
                                    )
                                    .unwrap();
                            }
                        }
                        comp_cve = comp.cve;
                        global_index += 1;
                        image_merge_end += 1;
                        comp_merge_end += 1;
                    }
                    image_cve += comp_cve;
                    if comp_merge_end > comp_merge_start {
                        sheet1
                            .merge_range(
                                comp_merge_start,
                                2,
                                comp_merge_end,
                                2,
                                comp_id,
                                Some(&format2),
                            )
                            .unwrap();

                        sheet1
                            .merge_range(
                                comp_merge_start,
                                3,
                                comp_merge_end,
                                3,
                                &comp_cve.to_string(),
                                Some(&format2),
                            )
                            .unwrap();

                        sheet1
                            .write_number(
                                comp_merge_start as u32,
                                3,
                                comp_cve as f64,
                                Some(&format2),
                            )
                            .unwrap();
                    }
                }

                if image_merge_end > image_merge_start {
                    sheet1
                        .merge_range(image_merge_start, 0, image_merge_end, 0, k, Some(&format2))
                        .unwrap();

                    sheet1
                        .merge_range(
                            image_merge_start,
                            1,
                            image_merge_end,
                            1,
                            &image_cve.to_string(),
                            Some(&format2),
                        )
                        .unwrap();
                }
                sheet1
                    .write_number(
                        image_merge_start as u32,
                        1,
                        image_cve as f64,
                        Some(&format2),
                    )
                    .unwrap();
            }
        }
    }
}

fn parse_object(
    workbook: &mut Excel,
    sheet: &str,
    object_map: &mut HashMap<String, HashMap<String, Vec<CveComponent>>>,
) {
    let range = workbook.worksheet_range(sheet).unwrap();
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut object_index: usize = 0;
    let mut vulnerability_index: usize = 0;
    let mut binary_object_index: usize = 0;

    let object_flags = vec!["dockerhub.kubekey.local", "docker.io"];

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
                    } else if val == COLUMN_FILED_BINARY_OBJECT {
                        binary_object_index = header_idx;
                    }
                }
            }
            // println!(
            //     "object_index:{:#?} component_index:{:#?} version_index:{:#?} vulnerability_index:{:#?} binary_object_index:{:#?}",
            //     object_index, component_index, version_index, vulnerability_index, binary_object_index
            // );
            if component_index == version_index
                || component_index == object_index
                || component_index == vulnerability_index
                || component_index == binary_object_index
                || version_index == object_index
                || version_index == vulnerability_index
                || version_index == binary_object_index
                || object_index == vulnerability_index
                || object_index == binary_object_index
                || vulnerability_index == binary_object_index
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
            let binary_object = match vals.get(binary_object_index) {
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
            let cve_component = CveComponent::new(
                format!("{}{}", component, version),
                binary_object.to_string(),
                vulnerability,
            );
            let object_val = match vals.get(object_index) {
                Some(v) => {
                    let v = match v {
                        DataType::String(v) => {
                            let mut object_key: String = String::new();
                            let secs: Vec<&str> = v.split('/').collect();
                            for &sec in secs.iter() {
                                for flag in object_flags.iter() {
                                    if sec.contains(*flag) {
                                        object_key = sec.to_string();
                                    }
                                }
                            }
                            if object_key.is_empty() {
                                if let Some(k) = secs.last() {
                                    object_key = k.to_string();
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

            let mut object: Vec<&str>;
            if object_val.contains('#') {
                object = object_val.split('#').collect();
            } else {
                object = object_val.split(' ').collect();
            }

            if let Some(object) = object.pop() {
                let object = object.trim_end_matches(".tar_");
                // println!(
                //     "object:{:#?}\ncomponent:{:#?}\nversion:{:#?}\nvulnerability:{}\ncve_component:{:#?}",
                //     object, component, version, vulnerability, cve_component
                // );
                if object.is_empty() || component.is_empty() || version.is_empty() {
                    continue;
                }
                if let Some(components) = object_map.get_mut(object) {
                    if let Some(binarys) = components.get_mut(&cve_component.id()) {
                        binarys.push(cve_component);
                    } else {
                        components.insert(cve_component.id(), vec![cve_component]);
                    }
                } else {
                    let mut comps: HashMap<String, Vec<CveComponent>> = HashMap::new();
                    comps.insert(cve_component.id(), vec![cve_component]);
                    object_map.insert(object.to_string(), comps);
                }
            }
        }
    }
}

fn write_component_output(component_map: &HashMap<String, Vec<String>>, out: &mut Workbook) {
    if !component_map.is_empty() {
        let format1 = set_title_format(out);
        let format2 = set_content_format(out);
        let mut sheet1 = out.add_worksheet(Some("component")).unwrap();
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

fn parse_component_cves(
    workbook: &mut Excel,
    sheet: &str,
    component_map: &mut HashMap<String, Vec<String>>,
    cve_map: &mut HashMap<String, String>,
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
            if let Some(cves) = component_map.get_mut(&component) {
                if !cves.contains(&cve_inner) {
                    cves.push(cve_inner.clone());
                }
            } else {
                component_map.insert(component.to_string(), vec![cve_inner.clone()]);
            }

            if !cve_map.contains_key(&cve_inner) {
                cve_map.insert(
                    cve_inner.clone(),
                    format!("https://avd.aliyun.com/detail?id={}", cve_inner),
                );
            }
        }
    }
}

fn write_cve_output(cve_map: &HashMap<String, String>, out: &mut Workbook, detail: bool) {
    if !cve_map.is_empty() {
        let format1 = set_title_format(out);
        let format2 = set_content_format(out);
        let mut sheet1 = out.add_worksheet(Some("cve")).unwrap();
        sheet1.write_string(0, 0, "cve", Some(&format1)).unwrap();
        sheet1.write_string(0, 1, "link", Some(&format1)).unwrap();
        sheet1.write_string(0, 2, "title", Some(&format1)).unwrap();
        sheet1
            .write_string(0, 3, "fix_label", Some(&format1))
            .unwrap();
        sheet1
            .write_string(0, 4, "publish", Some(&format1))
            .unwrap();
        sheet1
            .write_string(0, 5, "description", Some(&format1))
            .unwrap();
        sheet1
            .write_string(0, 6, "suggestion", Some(&format1))
            .unwrap();
        sheet1.write_string(0, 7, "score", Some(&format1)).unwrap();
        sheet1.write_string(0, 8, "effect", Some(&format1)).unwrap();

        let mut cve_keys: Vec<String> = cve_map.keys().map(|x| x.to_string()).collect();
        cve_keys.sort();

        for (index, k) in cve_keys.iter().enumerate() {
            if let Some(v) = cve_map.get(k) {
                sheet1
                    .write_string((index + 1) as u32, 0, k, Some(&format2))
                    .unwrap();
                sheet1
                    .write_url((index + 1) as u32, 1, v, Some(&format2))
                    .unwrap();
                if !detail {
                    // need parser CVE detail info
                    continue;
                }
                let ret = CVE_API.lock().unwrap().invoke("AliyunApi", k);
                // println!("{:#?}", ret.to_json());
                sheet1
                    .write_string((index + 1) as u32, 2, &ret.get("title"), Some(&format2))
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 3, &ret.get("fix_label"), Some(&format2))
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 4, &ret.get("publish"), Some(&format2))
                    .unwrap();
                sheet1
                    .write_string(
                        (index + 1) as u32,
                        5,
                        &ret.get("description"),
                        Some(&format2),
                    )
                    .unwrap();
                sheet1
                    .write_string(
                        (index + 1) as u32,
                        6,
                        &ret.get("suggestion"),
                        Some(&format2),
                    )
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 7, &ret.get("score"), Some(&format2))
                    .unwrap();
                sheet1
                    .write_string((index + 1) as u32, 8, &ret.get("effect"), Some(&format2))
                    .unwrap();
            }
        }
    }
}
