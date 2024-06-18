use std::fmt::format;
use crate::command::cve::api::base::HttpClient;
use crate::command::cve::lib::{Cve, CveApi};
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use serde::{self, Deserialize, Serialize};
use std::fs;
use std::io::Write;

pub const ALI_YUN_CVE_API: &str = "AliyunApi";

pub struct AliyunApi {
    http_client: HttpClient,
}

impl AliyunApi {
    pub fn new() -> AliyunApi {
        AliyunApi {
            http_client: HttpClient::new(),
        }
    }

    fn trim_node(&self, s: String) -> String {
        s.trim().trim_matches('\n').to_string()
    }

    fn parser_html(&self, target: &str) -> AliyunCve {
        let document = Document::from(target);
        let mut cve = AliyunCve::new();
        for node in document.find(Class("header__title__text")).take(1) {
            cve.title = self.trim_node(node.text());
        }
        for node in document.find(Class("metric-value")).take(1) {
            cve.id = self.trim_node(node.text());
        }

        for node in document.find(Class("metric-value")).take(3) {
            cve.fix_label = self.trim_node(node.text());
        }
        for node in document.find(Class("metric-value")).take(4) {
            cve.publish = self.trim_node(node.text());
        }
        for node in document
            .find(Class("text-detail").child(Name("div")))
            .collect::<Vec<Node>>()
        {
            cve.description += &format!("{}\n", self.trim_node(node.text()));
        }
        for node in document.find(Class("text-detail").and(Name("div"))).take(2) {
            cve.suggestion = self.trim_node(node.text());
        }
        for node in document
            .find(Class("cvss-breakdown").descendant(Class("cvss-breakdown__score")))
            .take(2)
        {
            cve.score = self.trim_node(node.text());
        }
        for node in document.find(Class("cvss-breakdown__desc")).take(1) {
            cve.effect = self.trim_node(node.text());
        }
        // println!("{:#?}", cve);
        cve
    }

    #[allow(unused)]
    fn write_file(&self, path: &str, response: reqwest::blocking::Response) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        file.write_all(&response.bytes().unwrap()).unwrap();
    }
}

impl Default for AliyunApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CveApi for AliyunApi {
    fn query(&self, id: &str) -> Box<dyn Cve> {
        let mut cve = AliyunCve::new();
        let resp = self
            .http_client
            .get(&(String::from("https://avd.aliyun.com/detail?id=") + id));
        match resp {
            Ok(v) => {
                cve = self.parser_html(v.text().unwrap().as_str());
            }
            Err(err) => {
                println!("{:#?}", err);
            }
        }
        Box::new(cve)
    }
    fn id(&self) -> String {
        String::from(ALI_YUN_CVE_API)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AliyunCve {
    id: String,
    title: String,
    fix_label: String,
    publish: String,
    description: String,
    suggestion: String,
    score: String,
    effect: String,
}

impl AliyunCve {
    fn new() -> AliyunCve {
        AliyunCve {
            id: String::new(),
            title: String::new(),
            fix_label: String::new(),
            publish: String::new(),
            description: String::new(),
            suggestion: String::new(),
            score: String::new(),
            effect: String::new(),
        }
    }
}

impl Cve for AliyunCve {
    fn to_json(&self) -> std::string::String {
        serde_json::to_string(&self).unwrap()
    }

    fn get(&self, key: &str) -> String {
        match key {
            "id" => self.id.clone(),
            "title" => self.title.clone(),
            "fix_label" => self.fix_label.clone(),
            "publish" => self.publish.clone(),
            "description" => self.description.clone(),
            "suggestion" => self.suggestion.clone(),
            "score" => self.score.clone(),
            "effect" => self.effect.clone(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_cve_api() {
        let cve_api = AliyunApi::new();
        cve_api.query("CVE-2023-25194");
    }

    #[test]
    fn test_cve_parser() {
        let _cve_api = AliyunApi::new();
        //cve_api.parser_html(include_str!("cve.html"));
    }
}
