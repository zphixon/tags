use crate::error;
use std::{fmt::Display, str::FromStr};

pub struct Tag {
    path: Vec<String>,
}

impl Tag {
    pub const SEP: &'static str = ".";
}

impl FromStr for Tag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stack: Vec<&str> = Vec::new();

        for component in s.split(Tag::SEP) {
            let component = component.trim();
            if component == "" {
                continue;
            }

            if component.bytes().any(|byte| !byte.is_ascii_graphic()) {
                return error!("Non-alphanumeric tag component {} disallowed", component);
            }

            stack.push(component);
        }

        if stack.is_empty() {
            return error!("Need at least one component in tag");
        }

        Ok(Tag {
            path: stack.into_iter().map(String::from).collect(),
        })
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join(Tag::SEP))
    }
}

#[cfg(test)]
#[test]
fn test() {
    panic!("{}", "a.hjeiwao..b,.c".parse::<Tag>().unwrap());
}
