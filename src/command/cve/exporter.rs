use std::collections::HashMap;
use std::fs;
use std::path::Path;
use clap::ArgMatches;
use xlsxwriter::Workbook;
use crate::command::cve::utils;


pub fn handler(matches: &ArgMatches) {
    let cve_input = matches.get_many::<String>("file").unwrap();
    let detail = matches.get_flag("detail");
    let path = matches.get_one::<String>("path").unwrap();
    let output = matches.get_one::<String>("output").unwrap();

    for (_, file) in cve_input.clone().enumerate() {
        if !Path::exists(Path::new(file)) {
            panic!("file not found {}", file)
        };
    }

    if !Path::exists(Path::new(path)) {
        fs::create_dir(path).unwrap();
    };

    let output = format!("{}/{}", path, output);
    let mut out = match Workbook::new(output.as_str()) {
        Ok(v) => v,
        Err(e) => {
            panic!("{}", e)
        }
    };

    let mut cve_ids: Vec<String> = Vec::new();
    for (_, file) in cve_input.enumerate() {
        let contents = fs::read_to_string(file).expect("Couldn't find or load that file.");
        let ids: Vec<String> = serde_json::from_str(&contents).unwrap();
        cve_ids.extend(ids)
    }

    let mut cve_map = HashMap::new();
    for (index, id) in cve_ids.iter().enumerate() {
        cve_map.insert(
            id.clone(),
            format!("https://avd.aliyun.com/detail?id={}", id),
        );
    }

    utils::write_cve_output(&cve_map, &mut out, detail);
}