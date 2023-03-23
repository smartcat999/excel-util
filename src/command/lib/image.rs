use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::Path,
    process::Command,
};

#[derive(Debug)]
pub enum CmdKind {
    InspectCmd,
}

pub trait Cmd {
    fn cmd(&self) -> String;
    fn handler(&mut self, s: &str);
    fn cmd_type(&self) -> CmdKind;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InspectCmd {
    image: String,
    cmd: String,
    layers: Vec<String>,
}

impl InspectCmd {
    pub fn new(image: &str) -> InspectCmd {
        InspectCmd {
            image: image.to_string(),
            cmd: format!(
                r#"skopeo inspect docker://{} --insecure-policy --tls-verify=false -f "{{{{.Layers}}}}""#,
                image
            ),
            layers: vec![],
        }
    }

    pub fn is_match(&self, layer: &str) -> bool {
        let mut is_match = false;
        for v in self.layers.iter() {
            if v.eq(layer) {
                is_match = true;
                return is_match;
            }
        }
        is_match
    }
}

impl Cmd for InspectCmd {
    fn cmd(&self) -> String {
        self.cmd.clone()
    }

    fn handler(&mut self, s: &str) {
        let s = s.trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '\n' || c == '\r');
        self.layers = s.split(' ').map(|x| x.to_string()).collect();
        println!("{:#?}", &self);
    }

    fn cmd_type(&self) -> CmdKind {
        CmdKind::InspectCmd
    }
}

pub fn run_inspect_cmd(cmd: &mut InspectCmd) {
    let input = cmd.cmd();
    let output = Command::new("bash").arg("-c").arg(&input).output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    // println!("cmd: {}\noutput: {}", input, &out);
    cmd.handler(&out);
}

fn multi_run_inspect_cmd(files: Vec<&str>) -> String {
    let mut image_layer: HashMap<String, InspectCmd> = HashMap::new();
    let target: Vec<&str> = if files.is_empty() {
        vec!["image.txt"]
    } else {
        files
    };
    // println!("{:#?}", target);
    for file in target {
        let f = file.trim();
        if !f.is_empty() {
            let data = std::fs::read_to_string(file).unwrap();
            let images: Vec<String> = data
                .trim()
                .split('\n')
                .map(|x| x.trim().trim_matches('\n').to_string())
                .collect();
            for image in images.iter() {
                let mut cmd = InspectCmd::new(image);
                run_inspect_cmd(&mut cmd);
                image_layer.insert(image.to_string(), cmd);
            }
        }
    }

    let v = serde_json::to_string(&image_layer);

    match v {
        Ok(v) => v,
        Err(e) => {
            println!("{:#?}", e);
            String::from("")
        }
    }
}

pub fn dump(input: Vec<&String>, path: &str, output: &str) {
    let image_layers = multi_run_inspect_cmd(input.iter().map(|x| x.as_str()).collect());
    if !image_layers.is_empty() {
        fs::write(Path::new(path).join(output), image_layers).unwrap();
    }
}

fn read_inspect_cmd_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, InspectCmd>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `InspectCmd`.
    let u = serde_json::from_reader(reader)?;

    // Return the `InspectCmd`.
    Ok(u)
}

pub fn load(files: Vec<&str>) -> HashMap<String, InspectCmd> {
    let mut image_layer: HashMap<String, InspectCmd> = HashMap::new();
    let target: Vec<&str> = if files.is_empty() {
        vec!["image.txt"]
    } else {
        files
    };

    for file in target {
        let f = file.trim();
        if !f.is_empty() {
            let tmp_layers = read_inspect_cmd_from_file(f);
            match tmp_layers {
                Ok(val) => {
                    for (k, v) in val {
                        image_layer.insert(k, v);
                    }
                },
                Err(e) => {
                    println!("{:#?}", e);
                }
            }
        }
    }
    image_layer
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    const INSPECT_CMD_STR: &str = "[sha256:fc251a6e798157dc3b46fd265da72f39cd848e3f9f4a0b28587d1713b878deb9 sha256:750218a4877112ef906ed0f0769642a12ffde1c6fed7ddebe3fe2090634e8684 sha256:57da6125df86d6738c5188290d51b700ee11ba2bc3ede18d2d415b2b33f70a36]";

    #[test]
    fn test_trim() {
        println!(
            "{}",
            INSPECT_CMD_STR.trim_matches(|c| c == '[' || c == ']' || c == ' ')
        );
    }

    #[test]
    fn test_cmd() {
        let mut cmd = InspectCmd::new("dockerhub.kubekey.local/huawei/ks-apiserver:v3.3.1-HW");
        println!("{:#?}", cmd.cmd_type());
        cmd.handler(INSPECT_CMD_STR);
    }

    #[test]
    fn test_run_cmd() {
        let mut cmd = InspectCmd::new("dockerhub.kubekey.local/huawei/ks-apiserver:v3.3.1-HW");
        run_inspect_cmd(&mut cmd);
    }

    #[test]
    fn test_multi_run_inspect_cmd() {
        multi_run_inspect_cmd(vec![]);
    }
}
