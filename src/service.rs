use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    #[serde(default, rename = "service")]
    services: HashMap<String, Service>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    image: String,
    #[serde(default = "default_port")]
    port: u16,

    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    labels: Vec<String>,
}

fn default_port() -> u16 {
    8080
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_example() {
        let input = r#"
            service "test" {
                image = "nginx:latest"
            }

            service "fake" {
                image = "fake:latest"
            }
        "#;

        let res: Definition = hcl::from_str(input).unwrap();
        let res = format!("{res:#?}");
        insta::assert_snapshot!(res);
    }
}
