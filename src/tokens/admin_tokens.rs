#![feature(proc_macro_hygiene, decl_macro)]

pub mod admin_tokens {
    use serde::{Deserialize, Serialize};
    use rocket_contrib::json::Json;

    use rusqlite::{params, Connection, Result};

    use rocket::Outcome;
    use rocket::http::Status;
    use rocket::request::{self, Request, FromRequest};



    fn find_admin_token_dao(token: String) -> bool {
        let mut conn = Connection::open("my.sqlite").unwrap();
        let mut stmt = match conn.prepare("SELECT token FROM admin_tokens WHERE token = :token") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };
        let mut rows = match stmt.query_named(&[(":token", &token)]) {
            Ok(i) => i,
            Err(i) => { println!("E {}", i.to_string()); panic!() }
        };

        while let Ok(row) = rows.next() {
            match row {
                Some(_) => return true,
                None => return false
            }
        }
        
        return false
    }

    fn list_admin_token_dao() -> Vec::<String> {
        let mut conn = Connection::open("my.sqlite").unwrap();
        let mut stmt = match conn.prepare("SELECT token FROM admin_tokens") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };
        let mut rows = match stmt.query_named(&[]) {
            Ok(i) => i,
            Err(i) => { println!("E {}", i.to_string()); panic!() }
        };

        let mut ret: Vec<String> = Vec::new();

        while let Ok(row) = rows.next() {
            match row {
                Some(i) => ret.push( match i.get(0) {
                    Ok(j) => j,
                    Err(j) => { println!("E {}", j.to_string()); panic!() }
                }),
                None => break
            }
        }
        
        ret
    }

    fn write_admin_token_dao(token: String) -> Result<bool, AdminTokenError> {
        let conn = Connection::open("my.sqlite").unwrap();

        match conn.execute(
            "INSERT INTO admin_tokens (token)
                    VALUES (?1)",
            params![token],
        ) {
            Ok(i) => (),
            Err(i) => { println!("D {}", i.to_string()); panic!() }
        }

        Ok(true)
    }

    fn delete_admin_token_dao(token: String) -> Result<bool, AdminTokenError> {
        let conn = Connection::open("my.sqlite").unwrap();

        match conn.execute(
            "DELETE FROM admin_tokens WHERE token = (?1)",
            params![token],
        ) {
            Ok(i) => (),
            Err(i) => { println!("D {}", i.to_string()); panic!() }
        }

        Ok(true)
    }
        
    #[derive(Serialize, Deserialize)]
    pub struct AdminTokenListWrapper {
        adminTokens: Vec<String>,
        version: String
    }

    // tokens controllers

    #[get("/admin/v1/list_admin_token")]
    pub fn list_admin_token_get(auth: AdminToken) -> Json<AdminTokenListWrapper>  {
        let ret = AdminTokenListWrapper {
            adminTokens: list_admin_token_dao(),
            version: "1.0.0".to_owned()
        };
        Json(ret)
    }

    #[get("/admin/v1/write_admin_token/<token>")]
    pub fn write_admin_token_get(token: String, auth: AdminToken) -> () {
        write_admin_token_dao(token);
    }

    #[get("/admin/v1/delete_admin_token/<token>")]
    pub fn delete_admin_token_get(token: String, auth: AdminToken) -> () {
        delete_admin_token_dao(token);
    }



    pub struct AdminToken(String);
    #[derive(Debug)]
    pub enum AdminTokenError {
        BadCount,
        Missing,
        Invalid,
    }
        
    use std::env;

    fn equals_super_admin_token(token: String) -> bool {
        let super_admin_token: Option<String> = match env::var("SUPER_ADMIN_TOKEN") {
            Ok(val) => Some(val),
            Err(e) => None,
        };

        match super_admin_token {
            Some(x) => token.to_string() == x,
            None => false
        }
    }

    impl<'a, 'r> FromRequest<'a, 'r> for AdminToken {
        type Error = AdminTokenError;

        fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
            //Outcome::Success(AdminToken("foo".to_string()))
            let keys: Vec<String> = request.headers()
                .get("Authorization")
                .filter_map(|x| { 
                    let tokens: Vec<&str> = x.split(" ").collect();
                    if tokens.len() != 2 {
                        return None
                    }
                    return Some(tokens[1].to_owned())
                })
                .collect();
            match keys.len() {
                0 => Outcome::Failure((Status::BadRequest, AdminTokenError::Missing)),
                1 if equals_super_admin_token(keys[0].to_string()) => Outcome::Success(AdminToken(keys[0].to_string())),
                1 if find_admin_token_dao(keys[0].to_string()) => Outcome::Success(AdminToken(keys[0].to_string())),
                1 => Outcome::Failure((Status::BadRequest, AdminTokenError::Invalid)),
                _ => Outcome::Failure((Status::BadRequest, AdminTokenError::BadCount)),
            }
        }
    }

}