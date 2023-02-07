use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default)]
//want to keep the name and description for future options
#[allow(dead_code)]
pub struct Snippet {
    name: String,
    prefix: String,
    description: String,
    body: String,
}

pub struct Snippets {
    snippets: HashMap<String, Snippet>,
}

impl Snippets {
    //want to keep the name and description for future options
    #[allow(dead_code)]
    fn get(&self, prefix: &str) -> &Snippet {
        self.snippets.get(prefix).unwrap()
    }
    fn get_body(&self, prefix: &str) -> String {
        let default = Snippet::default();
        self.snippets
            .get(prefix)
            .unwrap_or_else(|| &default)
            .body
            .to_string()
    }
}

#[derive(Parser)]
struct Cli {
    prefix: String,
}

fn main() {
    let args = Cli::parse();
    let home = dirs::home_dir()
        .unwrap()
        .join(".config/snippets/snippets.json");

    let snippets_file = std::fs::read_to_string(home)
        .expect("could not find `$HOME/.config/snippets/snippets.json`");
    let snippets = parse(&snippets_file);

    println!("{}", snippets.get_body(&args.prefix));
}

pub fn parse(json: &str) -> Snippets {
    let object: Value = serde_json::from_str(json).unwrap();
    let mut result: HashMap<String, Snippet> = HashMap::new();

    for element in object.as_object().unwrap().keys() {
        let obj = object.as_object().unwrap().get(element).unwrap();
        let snippet = convert_to_snippet(element, obj);
        result.insert(snippet.prefix.to_string(), snippet);
    }

    Snippets { snippets: result }
}

fn convert_to_snippet(element: &String, obj: &Value) -> Snippet {
    let snippet = Snippet {
        name: element.to_string(),
        prefix: obj["prefix"].as_str().unwrap().to_string(),
        body: extract_body(obj),
        description: obj["description"].as_str().unwrap_or_default().to_string(),
    };
    snippet
}

fn extract_body(obj: &Value) -> String {
    if obj["body"].is_array() {
        obj["body"]
            .as_array()
            .unwrap()
            .to_vec()
            .into_iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    } else {
        obj["body"].as_str().unwrap().to_string()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use crate::{parse, Snippet};

    #[test]
    fn should_add_a_function() {
        let snippet: Snippet = Snippet {
            name: "function".to_string(),
            prefix: "fn".to_string(),
            description: "creates a function".to_string(),
            body: "fn name() {\n\t\n}".to_string(),
        };
        let mut snippets: HashMap<String, Snippet> = HashMap::new();

        snippets.insert(snippet.prefix.to_string(), snippet);
        println!("{}", snippets.get("fn").unwrap().body);
        assert_eq!(snippets.get("fn").unwrap().body, "fn name() {\n\t\n}");
    }

    // That should work for visual studio code snippets
    #[test]
    #[ignore]
    fn should_replace_params() {
        let snippet: Snippet = Snippet {
            name: "function".to_string(),
            prefix: "fn".to_string(),
            description: "creates a function".to_string(),
            body: "fn ${2:params}() {\n\t\n}".to_string(),
        };
        let mut snippets: HashMap<String, Snippet> = HashMap::new();
        snippets.insert(snippet.prefix.to_string(), snippet);

        println!("{}", snippets.get("fn").unwrap().body);
        assert_eq!(snippets.get("fn").unwrap().body, "fn params() {\n\t\n}");
    }

    #[test]
    fn should_convert_snippet_from_json() {
        let json = r#"{
        "function": {
          "prefix": "fn",
          "body": "fn some() {}", 
          "description": "creates a function" 
        },
        "test": { 
          "prefix": "test", 
          "body": "\\#[test]\nfn should_do_something() {}"
        }
        }"#;

        let result = parse(&json);
        assert_eq!(result.get_body("fn"), "fn some() {}".to_string());
    }

    #[test]
    fn should_parse_body_when_is_an_array() {
        let json = r#"{
        "function": {
          "prefix": "fn",
          "body": ["fn some() {}"], 
          "description": "creates a function" 
        },
        "test": { 
          "prefix": "test", 
          "body": "\\#[test]\nfn should_do_something() {}"
        }
        }"#;

        let result = parse(&json);
        assert_eq!(result.get_body("fn"), "fn some() {}".to_string());
    }

    #[test]
    fn should_print_all_body_containing_more_than_one_string() {
        let json = r#"{
        "function": {
          "prefix": "fn",
          "body": ["fn", "some()", "{ }"], 
          "description": "creates a function" 
        },
        "test": { 
          "prefix": "test", 
          "body": "\\#[test]\nfn should_do_something() {}"
        }
        }"#;

        let result = parse(&json);
        assert_eq!(result.get_body("fn"), "fn some() { }".to_string());
    }

    #[test]
    fn should_return_empty_string_when_cant_find_the_prefix() {
        let json = r#"{
        "function": {
          "prefix": "fn",
          "body": ["fn", "some()", "{ }"], 
          "description": "creates a function" 
        },
        "test": { 
          "prefix": "test", 
          "body": "\\#[test]\nfn should_do_something() {}"
        }
        }"#;

        let result = parse(&json);
        assert_eq!(result.get_body("not_found"), "".to_string());
    }
}
