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
        let s = s.trim_matches('[').trim_matches(']');
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
