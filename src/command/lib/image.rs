use std::process::Command;

pub trait Cmd {
    fn cmd(&self) -> String;
    fn handler(&self, s: &str);
}

pub struct InspectCmd {
    cmd: String,
}

impl InspectCmd {
    pub fn new(name: &str) -> InspectCmd {
        InspectCmd {
            cmd: name.to_string(),
        }
    }
}

impl Cmd for InspectCmd {
    fn cmd(&self) -> String {
        format!(
            r#"skopeo inspect docker://{} --insecure-policy --tls-verify=false -f "{{{{.Layers}}}}""#,
            self.cmd
        )
    }

    fn handler(&self, s: &str) {
        let s = s.trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '\n' || c == '\r');
        println!("s: {}", s);
    }
}

pub fn run_cmd(cmd: Box<dyn Cmd>) {
    let input = cmd.cmd();
    let output = Command::new("bash").arg("-c").arg(&input).output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    println!("cmd: {}\noutput: {}", input, &out);
    cmd.handler(&out);
}


#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_trim() {
        let s = "[sha256:fc251a6e798157dc3b46fd265da72f39cd848e3f9f4a0b28587d1713b878deb9 sha256:750218a4877112ef906ed0f0769642a12ffde1c6fed7ddebe3fe2090634e8684 sha256:57da6125df86d6738c5188290d51b700ee11ba2bc3ede18d2d415b2b33f70a36]";
        println!("{}", s.trim_matches(|c| c == '[' || c == ']' || c == ' '));
    }
}