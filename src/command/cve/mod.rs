use std::{collections::HashMap, fs, path::Path, sync::Mutex, vec};

use clap::{value_parser, App, Arg, ArgAction, ArgMatches, Command};
use calamine::{open_workbook, DataType, Xlsx, Reader};
use xlsxwriter::{format::*, Workbook};

pub mod api;
pub mod lib;
pub mod utils;
pub mod exporter;
pub mod analyze;

use lib::CveApis;

use crate::command::lib::image;

use super::lib::image::ImageIndex;

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
        .subcommands(vec![
            Command::new("analyze")
                .about("分析CVE漏洞信息").arg(
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
                .arg(
                    Arg::new("release")
                        .long("release")
                        .action(clap::ArgAction::SetTrue)
                        .help("是否解析release包"),
                )
                .override_usage("etool cve analyze -p ./tmp -f Open_Source_Binary_Result.xlsx --sheet 组件报告 --sheet_ext 漏洞报告 --detail --release -o cve.xlsx\n  "),
            Command::new("export")
                .about("导出CVE漏洞库信息").arg(
                Arg::new("path")
                    .default_value("./tmp")
                    .short('p')
                    .help("生成的目标目录"),
            )
                .arg(
                    Arg::new("file")
                        .action(ArgAction::Append)
                        .value_parser(value_parser!(String))
                        .default_values(&["./cve.json"])
                        .short('f')
                        .help("待处理的Excel文件路径"),
                )
                .arg(
                    Arg::new("output")
                        .default_value("cve-export.xlsx")
                        .short('o')
                        .help("输出的Excel文件表格名称"),
                )
                .arg(
                    Arg::new("detail")
                        .long("detail")
                        .action(clap::ArgAction::SetTrue)
                        .help("是否输出CVE详细信息"),
                )
                .override_usage("etool cve export -p ./tmp -f cve.json --detail -o cve-export.xlsx\n  ")
        ]).override_usage("")
}

