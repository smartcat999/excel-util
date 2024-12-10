use std::collections::HashMap;
use xlsxwriter::{Format, Workbook, Worksheet};
use xlsxwriter::format::{FormatAlignment, FormatColor, FormatVerticalAlignment};
use crate::command::cve::{api, CVE_API, ALIYUN_CVE_API, TITLE_FONT_SIZE};

pub fn is_column_field_component(s: &str) -> bool {
    if s == "Component" || s == "组件名称" {
        return true;
    }
    false
}

pub fn is_column_field_version(s: &str) -> bool {
    if s == "Version" || s == "组件版本" {
        return true;
    }
    false
}

pub fn is_column_field_object(s: &str) -> bool {
    if s == "Object full path" || s == "文件路径" {
        return true;
    }
    false
}

pub fn is_column_field_vul_count(s: &str) -> bool {
    if s == "Vulnerability count" || s == "漏洞数量" {
        return true;
    }
    false
}

pub fn is_column_field_cve(s: &str) -> bool {
    if s == "CVE" || s == "CVE编号" {
        return true;
    }
    false
}

pub fn is_column_field_binary_object(s: &str) -> bool {
    if s =="Object" || s == "文件名" {
        return true;
    }
    false
}

pub fn set_title_format() -> Format {
    let mut format = Format::new();
    format.set_align(FormatAlignment::Center)
        .set_bg_color(FormatColor::Yellow)
        .set_font_size(TITLE_FONT_SIZE);
    format
}

pub fn set_content_format() -> Format {
    let mut format = Format::new();
    format.set_align(FormatAlignment::Center)
        .set_vertical_align(FormatVerticalAlignment::VerticalCenter);
    format
}

pub fn write_cve_output(cve_map: &HashMap<String, String>, out: &mut Workbook, detail: bool) {
    if !cve_map.is_empty() {
        let format1 = set_title_format();
        let format2 = set_content_format();


        let mut cve_keys: Vec<String> = cve_map.keys().map(|x| x.to_string()).collect();
        cve_keys.sort();

        let sheet_name = format!("cve-{}", 0);
        let mut sheet1: Worksheet = add_worksheet(out, sheet_name, &format1);
        for (mut index, k) in cve_keys.iter().enumerate() {
            if index % 60000 == 0 && index != 0 {
                let sheet_name = format!("cve-{}", index / 60000);
                sheet1 = add_worksheet(out, sheet_name, &format1);
            }
            index = index % 60000;
            if let Some(v) = cve_map.get(k) {
                sheet1
                    .write_string((index + 1) as u32, 0, k, Some(&format2))
                    .unwrap();
                println!("{}:{}", k, v);
                sheet1
                    .write_url((index + 1) as u32, 1, v, Some(&format2))
                    .unwrap();
                if !detail {
                    // need parser CVE detail info
                    continue;
                }
                let ret = CVE_API.lock().unwrap().invoke(api::aliyun_api::ALI_YUN_CVE_API, k);
                println!("{:#?}", ret.to_json());
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

pub async fn write_cve_output_async(cve_map: &HashMap<String, String>, out: &mut Workbook, detail: bool) {
    if !cve_map.is_empty() {
        let format1 = set_title_format();
        let format2 = set_content_format();


        let mut cve_keys: Vec<String> = cve_map.keys().map(|x| x.to_string()).collect();
        cve_keys.sort();

        let sheet_name = format!("cve-{}", 0);
        let mut sheet1: Worksheet = add_worksheet(out, sheet_name, &format1);
        for (mut index, k) in cve_keys.iter().enumerate() {
            if index % 60000 == 0 && index != 0 {
                let sheet_name = format!("cve-{}", index / 60000);
                sheet1 = add_worksheet(out, sheet_name, &format1);
            }
            index = index % 60000;
            if let Some(v) = cve_map.get(k) {
                sheet1
                    .write_string((index + 1) as u32, 0, k, Some(&format2))
                    .unwrap();
                println!("{}:{}", k, v);
                sheet1
                    .write_url((index + 1) as u32, 1, v, Some(&format2))
                    .unwrap();
                if !detail {
                    // need parser CVE detail info
                    continue;
                }
                let ret = ALIYUN_CVE_API.lock().unwrap().query(k).await;
                match ret {
                    Ok(ret) => {
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
                    },
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            }
        }
    }
}

fn add_worksheet<'a>(out: &'a mut Workbook, name: String, format: &'a Format) -> Worksheet<'a> {
    let mut sheet = out.add_worksheet(Some(&name)).unwrap();
    sheet.write_string(0, 0, "cve", Some(format)).unwrap();
    sheet.write_string(0, 1, "link", Some(format)).unwrap();
    sheet.write_string(0, 2, "title", Some(format)).unwrap();
    sheet
        .write_string(0, 3, "fix_label", Some(format))
        .unwrap();
    sheet
        .write_string(0, 4, "publish", Some(format))
        .unwrap();
    sheet
        .write_string(0, 5, "description", Some(format))
        .unwrap();
    sheet
        .write_string(0, 6, "suggestion", Some(format))
        .unwrap();
    sheet.write_string(0, 7, "score", Some(format)).unwrap();
    sheet.write_string(0, 8, "effect", Some(format)).unwrap();
    sheet
}