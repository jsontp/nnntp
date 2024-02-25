use std::collections::HashMap;

use jsontp::client::*;
use jsontp::server::*;

use clap::Parser;

use bcrypt::{hash, verify, DEFAULT_COST};

use rusqlite::{Connection, Result};

pub struct NnntpRequest {
    inner: JsontpRequest,
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

        let request = &self.inner;

        match request.body.other.get("nnntp") {
            Some(nnntp) => {
                match nnntp.get("type") {
                    Some(t) => {
                        match t.as_str() {
                            Some("post") => {
                                if nnntp.get("post").is_none() {
                                    return Err("post is required".to_string());
                                }

                                let post = nnntp.get("post").unwrap();

                                if post.get("subject").is_none() {
                                    return Err("subject is required".to_string());
                                }

                                if post.get("body").is_none() {
                                    return Err("body is required".to_string());
                                }

                                if nnntp.get("author").is_none() {
                                    return Err("author is required".to_string());
                                }

                                let author = nnntp.get("author").unwrap();

                                if author.get("username").is_none() {
                                    return Err("username is required".to_string());
                                }

                                if author.get("password").is_none() {
                                    return Err("password is required".to_string());
                                }

                                if author.get("email").is_none() {
                                    return Err("email is required".to_string());
                                }
                            },
                            Some("list") => {
                                // nothing to validate
                            },

                            Some("new") => {
                                if nnntp.get("username").is_none() {
                                    return Err("username is required".to_string());
                                }
                                if nnntp.get("password").is_none() {
                                    return Err("password is required".to_string());
                                }
                            },

                            Some("comment") => {
                                if nnntp.get("parent").is_none() {
                                    return Err("parent is required".to_string());
                                }
                                if nnntp.get("comment").is_none() {
                                    return Err("comment is required".to_string());
                                }
                                if nnntp.get("comment").unwrap().get("body").is_none() {
                                    return Err("body is required".to_string());
                                }

                                if nnntp.get("author").is_none() {
                                    return Err("author is required".to_string());
                                }
                                
                                let author = nnntp.get("author").unwrap();

                                if author.get("username").is_none() {
                                    return Err("username is required".to_string());
                                }

                                if author.get("password").is_none() {
                                    return Err("password is required".to_string());
                                }

                                if author.get("email").is_none() {
                                    return Err("email is required".to_string());
                                }
                            }
                            _ => {
                                return Err("valid type is required".to_string());
                            }
                        }
                    }
                    None => {
                        return Err("type is required".to_string());
                    }
                }
            }
            None => {
                return Err("nnntp is required".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Parser)]
struct Args {
    #[clap(long)]
    host: String,

    #[clap(long)]
    port: u16,
}

fn comment_on(
    parent_id: i32,
    body: &str,
    author: &str,
    password: &str,
    email: &str,
) -> Result<(), String> {
    if !verify_user(author, password).unwrap() {
        return Err("Invalid user".to_string());
    }

    let conn = Connection::open("posts.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            parent_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            author TEXT NOT NULL,
            author_email TEXT NOT NULL
        )",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO comments (parent_id, body, author, author_email) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![parent_id, body, author, email],
    ).unwrap();

    Ok(())
}

fn save_new_user(username: &str, password: &str) -> Result<(), String> {
    let conn = Connection::open("users.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    // first, check if the user already exists
    let mut stmt = conn
        .prepare("SELECT username FROM users WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([username]).unwrap();

    if rows.next().unwrap().is_some() {
        return Err("User already exists".to_string());
    }

    let hashed = hash(password, DEFAULT_COST).unwrap();
    conn.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        [username, &hashed],
    )
    .unwrap();

    Ok(())
}

fn verify_user(username: &str, password: &str) -> Result<bool, String> {
    let conn = Connection::open("users.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )
    .unwrap();
    let mut stmt = conn
        .prepare("SELECT password FROM users WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([username]).unwrap();

    let hashed: String = rows.next().unwrap().unwrap().get(0).unwrap();
    let result = verify(password, &hashed).unwrap();

    Ok(result)
}

fn post_to_group(
    group: &str,
    subject: &str,
    body: &str,
    author: &str,
    password: &str,
    email: &str,
) -> Result<i32, String> {
    if !verify_user(author, password).unwrap() {
        return Err("Invalid user".to_string());
    }

    let conn = Connection::open("posts.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            group_name TEXT NOT NULL,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            author TEXT NOT NULL,
            author_email TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO posts (group_name, subject, body, author, author_email) VALUES (?1, ?2, ?3, ?4, ?5)",
        [group, subject, body, author, email],
    )
    .unwrap();

    // now return the new post id
    let mut stmt = conn.prepare("SELECT id FROM posts WHERE group_name = ?1 AND subject = ?2 AND body = ?3 AND author = ?4").unwrap();
    let mut rows = stmt.query([group, subject, body, author]).unwrap();

    let id: i32 = rows.next().unwrap().unwrap().get(0).unwrap();

    Ok(id)
}

fn main() {
    let args: Args = Args::parse();

    let mut server = Server::new("NNNTP server", args.host, args.port);

    server.route("/comment",
        |req: JsontpRequest| {
            let nnntp_req = NnntpRequest::new(req);

            match nnntp_req.validate() {
                Ok(_) => {
                    let request = &nnntp_req.inner;
                    let nnntp = &request.body.other.get("nnntp").unwrap();
                    let parent_id = nnntp.get("parent").unwrap().get("id").unwrap().as_i64().unwrap() as i32;
                    let body = nnntp.get("comment").unwrap().get("body").unwrap().as_str().unwrap();
                    let author_obj = nnntp.get("author").unwrap();
                    let author = author_obj.get("username").unwrap().as_str().unwrap();
                    let password = author_obj.get("password").unwrap().as_str().unwrap();
                    let email = author_obj.get("email").unwrap().as_str().unwrap();

                    match comment_on(parent_id, body, author, password, email) {
                        Ok(_) => nnntp_req.inner.to_response(
                            Body::new("Commented OK", "identity", None),
                            200,
                            None,
                            Language::default(),
                            None,
                        ),
                        Err(e) => {
                            match e.as_str() {
                                "Invalid user" => nnntp_req.inner.to_response(
                                    Body::new("Invalid user", "identity", None),
                                    401,
                                    None,
                                    Language::default(),
                                    None,
                                ),
                                _ => nnntp_req.inner.to_response(
                                    Body::new("Failed to comment", "identity", None),
                                    400,
                                    None,
                                    Language::default(),
                                    None,
                                ),
                            }
                        }
                    }
                }
                Err(e) => nnntp_req.inner.to_response(
                    Body::new(format!("bad request - {}", e), "identity", None),
                    400,
                    None,
                    Language::default(),
                    None,
                ),
            }
        },
    );

    server.route("/post", |req: JsontpRequest| {
        let nnntp_req = NnntpRequest::new(req);
        match nnntp_req.validate() {
            Ok(_) => {
                let request = &nnntp_req.inner;
                let nnntp = match request.body.other.get("nnntp") {
                    Some(nnntp) => nnntp,
                    None => {
                        return nnntp_req.inner.to_response(
                            Body::new("", "identity", None),
                            400,
                            None,
                            Language::default(),
                            None,
                        )
                    }
                };
                
                let post = match nnntp.get("post") {
                    Some(post) => post,
                    None => {
                        return nnntp_req.inner.to_response(
                            Body::new("", "identity", None),
                            400,
                            None,
                            Language::default(),
                            None,
                        )
                    }
                };

                let subject = post.get("subject").unwrap().as_str().unwrap();
                let body = post.get("body").unwrap().as_str().unwrap();
                
                let author_obj = match nnntp.get("author") {
                    Some(author) => author,
                    None => {
                        return nnntp_req.inner.to_response(
                            Body::new("", "identity", None),
                            400,
                            None,
                            Language::default(),
                            None,
                        )
                    }
                };

                let author = author_obj.get("username").unwrap().as_str().unwrap();
                let password = author_obj.get("password").unwrap().as_str().unwrap();
                let email = author_obj.get("email").unwrap().as_str().unwrap();

                

                let group = request
                    .body
                    .other
                    .get("nnntp")
                    .unwrap()
                    .get("group")
                    .unwrap()
                    .as_str()
                    .unwrap();

                match post_to_group(group, subject, body, author, password, email) {
                    Ok(id) => {
                        let mut hs = HashMap::new();
                        hs.insert("id".to_string(), Value::Number(id.into()));
                        nnntp_req.inner.to_response(
                            Body::new("Posted OK", "identity", Some(hs)),
                            200,
                            None,
                            Language::default(),
                            None,
                        )
                    }
                    Err(e) => {
                        match e.as_str() {
                            "Invalid user" => nnntp_req.inner.to_response(
                                Body::new("Invalid user", "identity", None),
                                401,
                                None,
                                Language::default(),
                                None,
                            ),
                            _ => nnntp_req.inner.to_response(
                                Body::new("Failed to post", "identity", None),
                                400,
                                None,
                                Language::default(),
                                None,
                            ),
                        }
                    }
                }
            }
            Err(e) => nnntp_req.inner.to_response(
                Body::new(format!("bad request - {}", e), "identity", None),
                400,
                None,
                Language::default(),
                None,
            ),
        }
    });

    server.route("/new", |req| {
        let nnntp_req = NnntpRequest::new(req);

        match nnntp_req.validate() {
            Ok(_) => {
                let request = &nnntp_req.inner;
                let nnntp = &request.body.other.get("nnntp").unwrap();
                let username = nnntp.get("username").unwrap().as_str().unwrap();
                let password = nnntp.get("password").unwrap().as_str().unwrap();

                match save_new_user(username, password) {
                    Ok(_) => nnntp_req.inner.to_response(
                        Body::new("User created", "identity", None),
                        200,
                        None,
                        Language::default(),
                        None,
                    ),
                    Err(e) => nnntp_req.inner.to_response(
                        Body::new(e, "identity", None),
                        400,
                        None,
                        Language::default(),
                        None,
                    ),
                }
            }
            Err(e) => nnntp_req.inner.to_response(
                Body::new(format!("bad request - {}", e), "identity", None),
                400,
                None,
                Language::default(),
                None,
            ),
        }
    });

    server.route("/list", |req| {
        let conn = Connection::open("posts.db").unwrap();
        let mut stmt = conn.prepare("SELECT * FROM posts").unwrap();

        // do not use query_map because it returns a Result
        let rows = stmt.query([]).unwrap();


        let mut posts = vec![];

        for post in rows.mapped(
            |row| -> Result<(i32, String, String, String, String, String), rusqlite::Error> {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        ) {
            let (id, group, subject, body, author, email): (i32, String, String, String, String, String) =
                post.unwrap();

            if group != req.body.other.get("nnntp").unwrap().get("group").unwrap().as_str().unwrap() {
                continue;
            }

            let mut post = serde_json::map::Map::new();
            post.insert("id".to_string(), Value::Number(id.into()));
            post.insert("group_name".to_string(), Value::String(group));
            post.insert("subject".to_string(), Value::String(subject));
            post.insert("body".to_string(), Value::String(body));
            post.insert("author".to_string(), Value::String(author));
            post.insert("author_email".to_string(), Value::String(email));

            // now add the comments
            let mut stmt = conn.prepare("SELECT * FROM comments WHERE parent_id = ?1").unwrap();
            let rows = stmt.query([id]).unwrap();

            let mut comments = vec![];

            for comment in rows.mapped(
                |row| -> Result<(i32, String, String, String), rusqlite::Error> {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                },
            ) {
                let (_, body, author, email): (i32, String, String, String) = comment.unwrap();

                let mut comment = serde_json::map::Map::new();
                comment.insert("body".to_string(), Value::String(body));
                comment.insert("author".to_string(), Value::String(author));
                comment.insert("author_email".to_string(), Value::String(email));

                comments.push(Value::Object(comment));
            }

            post.insert("comments".to_string(), Value::Array(comments));

            posts.push(post);
        }

        let mut prepared_other: HashMap<String, Value> = HashMap::new();

        prepared_other.insert(
            "nnntp".to_string(),
            Value::Array(posts.iter().map(|x| Value::Object(x.clone())).collect()),
        );

        req.to_response(
            Body::new("processed OK", "identity", Some(prepared_other)),
            200,
            None,
            Language::default(),
            None,
        )
    });

    server.start();
}
