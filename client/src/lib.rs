use jsontp::client::*;

/*
format:
    {
        "type": "post", // or list, or comment
        "group": "comp.lang.rust", // group name

        if (type == "post") {
            "post": {
                "subject": "This is a subject",
                "body": "This is a body",
            },

            "author": {
                "username": "username",
                "password": "password",
                "email": "email",
            }
        }

        if (type == "comment") {
            "parent": {
                "id": "1234",
            },

            "comment": {
                "body": "This is a comment",
            },

            "author": {
                "username": "username",
                "password": "password",
                "email": "email",
            }
        }
    }

    it then returns the new post id (example: 1234) or the list of posts (example: [{id: 1234, subject: "This is a subject", body: "This is a body", author: "username"}]
*/

#[derive(Debug, Clone)]
pub struct Post {
    pub id: i32,
    pub subject: String,
    pub body: String,
    pub author: String,
    pub author_email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Posts {
    pub posts: Vec<Post>,

    pub group: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: Option<String>,

    pub password: String,
}

impl User {
    pub fn new<T: ToString>(username: T, email: Option<T>, password: T) -> User {
        User {
            username: username.to_string(),
            email: match email {
                Some(email) => Some(email.to_string()),
                None => None,
            },
            password: password.to_string(),
        }
    }

}

pub struct ServerConnection {
    pub host: String,
    pub port: u16,

    pub user: Option<User>,
}
impl ServerConnection {
    pub fn new<T: ToString>(host: T, port: u16, user: Option<User>) -> ServerConnection {
        ServerConnection {
            host: host.to_string(),
            port,
            user
        }
    }

    pub fn post<T: ToString>(&self, group: T, subject: T, body: T) -> Result<(), String> {
        let author = match self.user.clone() {
            Some(author) => author,
            None => return Err("No user provided".to_string()),
        };

        let client = Request::new()
            .body_key(
                "nnntp",
                Value::Object(
                    [
                        ("type".to_string(), Value::String("post".to_string())),
                        ("group".to_string(), Value::String(group.to_string())),
                        (
                            "post".to_string(),
                            Value::Object(
                                [
                                    ("subject".to_string(), Value::String(subject.to_string())),
                                    ("body".to_string(), Value::String(body.to_string())),
                                ]
                                .iter()
                                .cloned()
                                .collect(),
                            ),
                        ),
                        (
                            "author".to_string(),
                            Value::Object(
                                [
                                    (
                                        "username".to_string(),
                                        Value::String(author.username.clone()),
                                    ),
                                    (
                                        "password".to_string(),
                                        Value::String(author.password.clone()),
                                    ),
                                    (
                                        "email".to_string(),
                                        Value::String(
                                            author
                                                .email
                                                .clone()
                                                .unwrap_or("no_email@provided.com".to_string()),
                                        ),
                                    ),
                                ]
                                .iter()
                                .cloned()
                                .collect(),
                            ),
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                ),
            )
            .resource("/post");

        let response = client.send(self.host.clone(), self.port).unwrap();

        match response.status.code {
            200 => Ok(()),
            400 => Err("Invalid request".to_string()),
            401 => Err("Unauthorized".to_string()),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn list<T: ToString>(&self, group: T) -> Result<Posts, String> {
        let client = Request::new()
            .body_key(
                "nnntp",
                Value::Object(
                    [
                        ("type".to_string(), Value::String("list".to_string())),
                        ("group".to_string(), Value::String(group.to_string())),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                ),
            )
            .resource("/list");

        let response: Result<JsontpResponse, String> = client.send(self.host.clone(), self.port);

        let other = match response {
            Ok(response) => response.body.other,
            Err(err) => return Err(err),
        };

        let posts = match other.get("nnntp") {
            Some(Value::Array(posts)) => Ok(Value::Array(posts.clone())),
            _ => Err("Invalid response".to_string()),
        };

        let group_name = match posts.clone() {
            Ok(Value::Array(posts)) => posts
                .get(0)
                .unwrap_or(&Value::String("no_posts".to_string()))
                .get("group_name")
                .unwrap_or(&Value::String("no_posts".to_string()))
                .as_str()
                .unwrap_or("no_posts")
                .to_string(),
            _ => return Err("Invalid response".to_string()),
        };

        let mut posts_instance = Posts {
            posts: vec![],
            group: group_name,
        };

        for post in posts.unwrap_or(Value::Array(vec![])).as_array().unwrap() {
            let post = match post {
                Value::Object(post) => post,
                _ => return Err("Invalid response".to_string()),
            };

            let id = match post.get("id") {
                Some(Value::Number(id)) => id.clone(),
                _ => return Err("Invalid response".to_string()),
            };

            let subject = match post.get("subject") {
                Some(Value::String(subject)) => subject.clone(),
                _ => return Err("Invalid response".to_string()),
            };

            let body = match post.get("body") {
                Some(Value::String(body)) => body.clone(),
                _ => return Err("Invalid response".to_string()),
            };

            let author = match post.get("author") {
                Some(Value::String(author)) => author.clone(),
                _ => return Err("Invalid response".to_string()),
            };

            let author_email = match post.get("author_email") {
                Some(Value::String(author_email)) => Some(author_email.clone()),
                _ => None,
            };

            posts_instance.posts.push(Post {
                id: id.to_string().parse().unwrap(),
                subject,
                body,
                author,
                author_email
            });
        }

        Ok(posts_instance)
    }

    pub fn new_user<T: ToString>(&self, username: T, password: T) -> Result<(), String> {
        let client = Request::new()
            .body_key(
                "nnntp",
                Value::Object(
                    [
                        ("type".to_string(), Value::String("new".to_string())),
                        ("username".to_string(), Value::String(username.to_string())),
                        ("password".to_string(), Value::String(password.to_string())),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                ),
            )
            .resource("/new");

        let response = client.send(self.host.clone(), self.port).unwrap();

        match response.status.code {
            200 => Ok(()),
            400 => Err("Invalid request".to_string()),
            401 => Err("Unauthorized".to_string()),
            _ => Err("Unknown error".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let user = User::new("username", None, "password");

        let server = ServerConnection::new("localhost", 8080, Some(user));

        println!("Posting: {:?}", server.post("comp.lang.rust", "This doesn't work", "This is a body"));

        println!("Listing: {:?}", server.list("comp.lang.rust"));
    }
}
