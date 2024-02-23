use std::collections::HashMap;

use jsontp::server::*;
use jsontp::client::*;

pub struct NnntpRequest {
    inner: JsontpRequest
}

impl NnntpRequest {
    fn new(inner: JsontpRequest) -> NnntpRequest {
        NnntpRequest { inner }
    }

    fn validate(&self) -> Result<(), String> {
        /*
        format:
            {
                "type": "post", // or list, or comment
                "group": "comp.lang.rust", // group name
                
                if (type == "post") {
                    "subject": "This is a subject",
                    "body": "This is a body",
                    "author": "author",
                }

                if (type == "comment") {
                    "parent": {
                        "subject": "This is a subject",
                        "author": "author",
                    },
                    "body": "This is a body",
                }
            }
        */

        let request = &self.inner;

        match request.body.other.get("nnntp") {
            Some(nnntp) => {
                match nnntp.get("type") {
                    Some(t) => {
                        match t.as_str() {
                            Some("post") => {
                                if nnntp.get("subject").is_none() {
                                    return Err("subject is required".to_string());
                                }
                                if nnntp.get("body").is_none() {
                                    return Err("body is required".to_string());
                                }
                                if nnntp.get("author").is_none() {
                                    return Err("author is required".to_string());
                                }
                            },
                            Some("comment") => {
                                if nnntp.get("parent").is_none() {
                                    return Err("parent is required".to_string());
                                }
                                if nnntp.get("body").is_none() {
                                    return Err("body is required".to_string());
                                }
                            },
                            Some("list") => {
                                // nothing to validate
                            },
                            _ => {
                                return Err("valid type is required".to_string());
                            }
                        }
                    },
                    None => {
                        return Err("type is required".to_string());
                    }
                }
            },
            None => {
                return Err("nnntp is required".to_string());
            }
        }

        Ok(())
    }
}

struct NnntpBuilder {
    inner: Request
}

impl NnntpBuilder {
    fn new() -> NnntpBuilder {
        let 
        inner.inner.body.other.insert("nnntp".to_string(), Value::Object)(serde_json::Map::new())
        
    }

    fn set_type(&mut self, t: &str) -> &mut NnntpBuilder {
        self.inner.body_key("nnntp", Value::Object(serde_json::Map::new()));

        self
    }


}