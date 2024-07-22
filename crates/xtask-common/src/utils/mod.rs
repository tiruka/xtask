use std::process::Command;

pub mod cargo;
pub mod mdbook;
pub mod process;
pub mod prompt;
pub mod rustup;
pub mod time;
pub mod workspace;

#[derive(Clone)]
pub struct Params {
    pub params: Vec<String>,
}

impl<const N: usize> From<[&str; N]> for Params {
    fn from(value: [&str; N]) -> Self {
        Self {
            params: value.iter().map(|v| v.to_string()).collect(),
        }
    }
}

impl From<&str> for Params {
    fn from(value: &str) -> Self {
        Self {
            params: vec![value.to_string()],
        }
    }
}

impl From<Vec<&str>> for Params {
    fn from(value: Vec<&str>) -> Self {
        Self {
            params: value.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.params.join(" ").as_str())
    }
}

impl<Rhs: Into<Params>> std::ops::Add<Rhs> for Params {
    type Output = Params;

    fn add(mut self, rhs: Rhs) -> Self::Output {
        let rhs: Params = rhs.into();
        self.params.extend(rhs.params);
        self
    }
}

pub fn get_command_line_from_command(command: &Command) -> String {
    let args: Vec<String> = command
        .get_args()
        .map(|arg| format!("\"{}\"", arg.to_string_lossy().into_owned()))
        .collect();
    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        args.join(" ")
    )
}
