use std::collections::HashMap;
use std::fs;
use std::path::Path;
use calamine::{DataType, open_workbook, Reader, Xlsx};
use clap::ArgMatches;
use xlsxwriter::{Format, Workbook};
use xlsxwriter::format::{FormatAlignment, FormatColor, FormatVerticalAlignment};
use crate::command::cve::{TITLE_FONT_SIZE, utils};
use crate::command::lib::image;
use crate::command::lib::image::ImageIndex;

pub fn handler(matches: &ArgMatches) {
    let path = matches.get_one::<String>("path").unwrap();
    let files: Vec<&String> = matches
        .get_many::<String>("file")
        .unwrap()
        .collect::<Vec<&String>>();
    let sheet = matches.get_one::<String>("sheet").unwrap();
    let sheet_ext = matches.get_one::<String>("sheet_ext").unwrap();
    let detail = matches.get_flag("detail");
    let release = matches.get_flag("release");
    let output = matches.get_one::<String>("output").unwrap();

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    // output
    let output = format!("{}/{}", path, output);
    let mut out = match Workbook::new(output.as_str()) {
        Ok(v) => v,
        Err(e) => {
            panic!("{}", e)
        }
    };

    let mut object_map: HashMap<String, HashMap<String, Vec<CveComponent>>> = HashMap::new();
    let mut component_map: HashMap<String, Vec<Cve>> = HashMap::new();
    let mut cve_map: HashMap<String, String> = HashMap::new();
    let mut image_index: ImageIndex = image::ImageIndex::new(HashMap::new());

    if release {
        image_index = image::load(vec!["./tmp/image.json"]);
    }

    for file in files.iter() {
        let mut workbook: Xlsx<_> = open_workbook(file).unwrap();

        // parse component's cve
        let component_sheet = workbook.worksheet_range(sheet_ext).unwrap();
        parse_component_cves(&component_sheet, &mut component_map, &mut cve_map);

        // parse object's component
        let object_sheet = workbook.worksheet_range(sheet).unwrap();
        parse_object(&object_sheet, &mut object_map, release, &image_index);
    }

    write_component_output(&component_map, &mut out);
    write_object_output(&object_map, &component_map, &mut out);
    utils::write_cve_output(&cve_map, &mut out, detail);
    println!(
        "object num: {:?}\ncomponent num: {:?}\ncve num: {:?}",
        object_map.len(),
        component_map.len(),
        cve_map.len()
    );
    println!("inuput: {:#?}\noutput: {:#?}", files, output);
}

// format init


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

impl Clone for CveComponent {
    fn clone(&self) -> CveComponent {
        CveComponent {
            component: self.component.clone(),
            binary: self.binary.clone(),
            cve: self.cve,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Cve {
    cve: String,
    binary: String,
}

impl Cve {
    fn new(cve: String, binary: String) -> Cve {
        Cve { cve, binary }
    }
}

impl PartialEq for Cve {
    fn eq(&self, other: &Self) -> bool {
        return self.cve == other.cve && self.binary == other.binary;
    }

    fn ne(&self, other: &Self) -> bool {
        return self.cve != other.cve || self.binary != other.binary;
    }
}

fn write_object_output(
    object_map: &HashMap<String, HashMap<String, Vec<CveComponent>>>,
    component_map: &HashMap<String, Vec<Cve>>,
    out: &mut Workbook,
) {
    if !object_map.is_empty() {
        let format1 = utils::set_title_format();
        let format2 = utils::set_content_format();
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

        sheet1.write_string(0, 6, "path", Some(&format1)).unwrap();

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
                    let mut comp_cve_num = 0;
                    for comp in comps.iter() {
                        if component_map.contains_key(&comp.id()) {
                            if let Some(comp_cves) = component_map.get(&comp.id()) {
                                for (_, cve_detail) in comp_cves.iter().filter(|a| a.binary.contains(k) && a.binary.ends_with(format!("/{}", &comp.binary).as_str())).collect::<Vec<&Cve>>().iter().enumerate() {
                                    sheet1
                                        .write_string(global_index as u32, 0, k, Some(&format2))
                                        .unwrap();
                                    sheet1
                                        .write_string(global_index as u32, 2, &comp.id(), Some(&format2))
                                        .unwrap();
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
                                    sheet1
                                        .write_string(
                                            global_index as u32,
                                            4,
                                            cve_detail.cve.as_str(),
                                            Some(&format2),
                                        )
                                        .unwrap();
                                    sheet1
                                        .write_string(
                                            global_index as u32,
                                            6,
                                            cve_detail.binary.as_str(),
                                            Some(&format2),
                                        )
                                        .unwrap();
                                    global_index += 1;
                                    image_merge_end += 1;
                                    comp_merge_end += 1;
                                    comp_cve_num += 1
                                }
                            }
                        }
                    }
                    image_cve += comp_cve_num;
                    if comp_merge_end > comp_merge_start {
                        // sheet1
                        //     .merge_range(
                        //         comp_merge_start,
                        //         2,
                        //         comp_merge_end,
                        //         2,
                        //         comp_id,
                        //         Some(&format2),
                        //     )
                        //     .unwrap();

                        // sheet1
                        //     .merge_range(
                        //         comp_merge_start,
                        //         3,
                        //         comp_merge_end,
                        //         3,
                        //         &comp_cve.to_string(),
                        //         Some(&format2),
                        //     )
                        //     .unwrap();

                        sheet1
                            .write_number(
                                comp_merge_start as u32,
                                3,
                                comp_cve_num as f64,
                                Some(&format2),
                            )
                            .unwrap();
                    }
                }

                if image_merge_end > image_merge_start {
                    // sheet1
                    //     .merge_range(image_merge_start, 0, image_merge_end, 0, k, Some(&format2))
                    //     .unwrap();

                    // sheet1
                    //     .merge_range(
                    //         image_merge_start,
                    //         1,
                    //         image_merge_end,
                    //         1,
                    //         &image_cve.to_string(),
                    //         Some(&format2),
                    //     )
                    //     .unwrap();
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
    sheet: &calamine::Range<DataType>,
    object_map: &mut HashMap<String, HashMap<String, Vec<CveComponent>>>,
    release: bool,
    image_index: &ImageIndex,
) {
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut object_index: usize = 0;
    let mut vulnerability_index: usize = 0;
    let mut binary_object_index: usize = 0;

    let scan_object_flags = vec!["dockerhub.kubekey.local", "docker.io", "scan.tar.gz"];
    let release_object_flags = vec!["blobs"];

    for (index, vals) in sheet.rows().enumerate() {
        if index == 0 {
            for (header_idx, header_content) in vals.iter().enumerate() {
                if let calamine::DataType::String(val) = header_content {
                    if utils::is_column_field_component(&val) {
                        component_index = header_idx;
                    } else if utils::is_column_field_version(&val) {
                        version_index = header_idx;
                    } else if utils::is_column_field_object(&val) {
                        object_index = header_idx;
                    } else if utils::is_column_field_vul_count(&val) {
                        vulnerability_index = header_idx;
                    } else if utils::is_column_field_binary_object(&val) {
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
                    _ => "0",
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
                    _ => "",
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
                    _ => "",
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
                    _ => "",
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
                            let object_flags: Vec<&str> = if release {
                                release_object_flags.clone()
                            } else {
                                scan_object_flags.clone()
                            };

                            for (index, &sec) in secs.iter().enumerate() {
                                for flag in object_flags.iter() {
                                    if sec.contains(*flag) {
                                        object_key = if release {
                                            String::from("sha256:")
                                                + secs[index + 2].to_string().trim_matches('_')
                                        } else if *flag == "scan.tar.gz" {
                                            secs[index + 3].to_string().trim_matches('_').to_string()
                                        } else {
                                            sec.to_string()
                                        };
                                        break;
                                    } else if !release && sec.contains("images") {
                                        object_key = secs[index + 1].to_string();
                                        break;
                                    }
                                }
                            }
                            if object_key.is_empty() {
                                object_key = v.to_string()
                            }
                            object_key
                        }
                        DataType::Int(_)
                        | DataType::Float(_)
                        | DataType::Bool(_)
                        | DataType::Error(_)
                        | DataType::Empty => String::from(""),
                        _ => String::from(""),
                    };
                    v
                }
                None => String::from(""),
            };

            let mut object_bytes: Vec<&str>;
            let mut object_key: &str = "";
            if object_val.contains('#') {
                object_bytes = object_val.split('#').collect();
                if let Some(object) = object_bytes.pop() {
                    object_key = object.trim_end_matches(".tar_");
                }
            } else if object_val.starts_with("sha256:") {
                object_key = object_val.as_str();
            } else {
                object_bytes = object_val.split(' ').collect();
                if let Some(object) = object_bytes.pop() {
                    object_key = object.trim_end_matches(".tar_").trim_end_matches(".tar.gz");
                }
            }

            // println!(
            //     "object:{:#?}\ncomponent:{:#?}\nversion:{:#?}\nvulnerability:{}\ncve_component:{:#?}",
            //     object_key, component, version, vulnerability, cve_component
            // );
            if object_key.is_empty() || component.is_empty() || version.is_empty() {
                continue;
            }
            if object_key.starts_with("sha256:") {
                let object_keys = image_index.search_layers(object_key.to_string());
                println!("images: {:#?}", object_keys);
                for (_, object_key) in object_keys.iter().enumerate() {
                    update_object_cve_component(object_map, object_key, cve_component.clone())
                }
            } else {
                update_object_cve_component(object_map, object_key, cve_component)
            }
        }
    }
}

fn update_object_cve_component(object_map: &mut HashMap<String, HashMap<String, Vec<CveComponent>>>,
                               object_key: &str,
                               cve_component: CveComponent) {
    if let Some(components) = object_map.get_mut(object_key) {
        if let Some(binarys) = components.get_mut(&cve_component.id()) {
            binarys.push(cve_component);
        } else {
            components.insert(cve_component.id(), vec![cve_component]);
        }
    } else {
        let mut comps: HashMap<String, Vec<CveComponent>> = HashMap::new();
        comps.insert(cve_component.id(), vec![cve_component]);
        object_map.insert(object_key.to_string(), comps);
    }
}

fn write_component_output(component_map: &HashMap<String, Vec<Cve>>, out: &mut Workbook) {
    if !component_map.is_empty() {
        let format1 = utils::set_title_format();
        let format2 = utils::set_content_format();
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
                    .write_string((index + 1) as u32, 1, v.iter().map(|a| a.clone().cve).collect::<Vec<String>>().join("\n").as_str(), Some(&format2))
                    .unwrap();
                sheet1
                    .write_number((index + 1) as u32, 2, v.len() as f64, Some(&format2))
                    .unwrap();
            }
        }
    }
}

fn parse_component_cves(
    sheet: &calamine::Range<DataType>,
    component_map: &mut HashMap<String, Vec<Cve>>,
    cve_map: &mut HashMap<String, String>,
) {
    let mut component_index: usize = 0;
    let mut version_index: usize = 0;
    let mut cve_index: usize = 0;
    let mut object_index: usize = 0;

    for (index, vals) in sheet.rows().enumerate() {
        if index == 0 {
            for (header_idx, header_content) in vals.iter().enumerate() {
                if let calamine::DataType::String(val) = header_content {
                    if utils::is_column_field_component(&val) {
                        component_index = header_idx;
                    } else if utils::is_column_field_version(&val) {
                        version_index = header_idx;
                    } else if utils::is_column_field_cve(&val) {
                        cve_index = header_idx;
                    } else if utils::is_column_field_object(&val) {
                        object_index = header_idx;
                    }
                }
            }
            // println!(
            //     "component_index:{:#?} version_index:{:#?} cve_index:{:#?}",
            //     component_index, version_index, cve_index
            // );
            if component_index == version_index
                || component_index == cve_index
                || component_index == object_index
                || version_index == cve_index
                || version_index == object_index
                || cve_index == object_index
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
                    _ => "",
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
                    _ => "",
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
                    _ => "",
                },
                None => "",
            };
            let mut object = match vals.get(object_index) {
                Some(v) => match v {
                    DataType::String(v) => {
                        v
                    }
                    _ => "",
                },
                None => "",
            };
            let object_path= object.split('/').enumerate()
                .filter(|&(i, _)| i == 3 || i >= 6)
                .map(|(_, e)| e)
                .collect::<Vec<&str>>().join("/").to_string();
            object = &object_path;
            if cve.is_empty() || component.is_empty() || version.is_empty() || object.is_empty() {
                continue;
            }
            let component = format!("{}{}", component, version);
            let cve_inner = Cve::new(cve.to_string(), object.to_string());
            if let Some(cves) = component_map.get_mut(&component) {
                if !cves.contains(&cve_inner) {
                    cves.push(cve_inner.clone());
                }
            } else {
                component_map.insert(component.to_string(), vec![cve_inner.clone()]);
            }

            if !cve_map.contains_key(&cve_inner.cve) {
                cve_map.insert(
                    cve_inner.cve.clone(),
                    format!("https://avd.aliyun.com/detail?id={}", cve_inner.cve),
                );
            }
        }
    }
}

