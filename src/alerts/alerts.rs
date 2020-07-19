#![feature(proc_macro_hygiene, decl_macro)]

pub mod alerts {
    use std::collections::HashMap;
    use std::cmp::max;
    use serde::{Deserialize, Serialize};
    use rocket_contrib::json::Json;
    use chrono::{DateTime, TimeZone, Utc};
    
    use rusqlite::{params, Connection, Result};

    use crate::tokens::admin_tokens;
    use admin_tokens::admin_tokens::*;
    use crate::tokens::alerter_tokens;
    use alerter_tokens::alerter_tokens::*;


    

    fn read_alert_list_dao() -> Result<AlertListWrapper, rusqlite::Error> {
        let mut conn = Connection::open("my.sqlite").unwrap();
        let mut stmt = match conn.prepare("SELECT id, timestamp FROM alerts") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };

        let AlertInstances = stmt.query_map(params![], |row| {
            let ts: i64 = match row.get(1) {
                Ok(i) => i,
                Err(i) => { println!("B {}", i.to_string()); panic!() }
            };
            Ok(AlertInstance {
                id: match row.get(0) {
                    Ok(i) => i,
                    Err(i) => { println!("C {}", i.to_string()); panic!() }
                },
                timestamp: Utc.timestamp(ts, 0),
            })
        })?;

        let mut most_recent_alerts: HashMap<String, DateTime<Utc>> = HashMap::new();

        for AlertInstance in AlertInstances {
            let x = match AlertInstance {
                Ok(i) => { 
                    match most_recent_alerts.get(&i.id) {
                        Some(j) => {
                            let tmp = max(i.timestamp.timestamp(), j.timestamp());
                            most_recent_alerts.insert(i.id, Utc.timestamp(tmp, 0));
                        },
                        None => {
                            most_recent_alerts.insert(i.id, i.timestamp);
                        }    
                    };
                },
                Error => ()
            };
        }

        let ret = most_recent_alerts.iter()
            .map(|x| AlertInstance{ id: x.0.to_owned(), timestamp: x.1.to_owned() } )
            .collect::<Vec<AlertInstance>>();

            
        let list = AlertListWrapper{ 
            alerts: ret,
            version: "1.0.0".to_owned()
        };

        Ok(list)
    }

    fn read_alerts_dao(id: String) -> Result<AlertWrapper, rusqlite::Error> {
        let mut conn = Connection::open("my.sqlite").unwrap();
            let mut stmt = match conn.prepare("SELECT id, timestamp FROM alerts WHERE id = :id") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };
        let mut rows = match stmt.query_named(&[(":id", &id)]) {
            Ok(i) => i,
            Err(i) => { println!("E {}", i.to_string()); panic!() }
        };

        let mut v: Vec<DateTime<Utc>> = Vec::new();

        while let Ok(row) = rows.next() {
            match row {
                Some(i) => {
                    let timestamp: i64 = match i.get(1) {
                        Ok(j) => j,
                        Err(j) => { println!("F {}", j.to_string()); panic!() }
                    };
                    v.push(Utc.timestamp(timestamp,0));
                },
                None => break
            }
        }
        let a1 = Alert{ id: id, fired_timestamps: v };
        Ok(AlertWrapper { alert: Some(a1), version: "1.0.0".to_owned() })
    }

    fn write_alert_dao(a: &AlertInstance) -> Result<&AlertInstance, rusqlite::Error> {
        let conn = Connection::open("my.sqlite").unwrap();

        match conn.execute(
            "INSERT INTO alerts (id, timestamp)
                    VALUES (?1, ?2)",
            params![a.id, a.timestamp.timestamp()],
        ) {
            Ok(i) => Ok(a),
            Err(i) => { println!("D {}", i.to_string()); panic!() }
        }

    }

    fn register_find_any(id: String) -> bool {
        let conn = Connection::open("my.sqlite").unwrap();
        let mut stmt = match conn.prepare("SELECT id, timestamp FROM alerts WHERE id = :id") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };
        let mut rows = match stmt.query_named(&[(":id", &id)]) {
            Ok(i) => i,
            Err(i) => { println!("E {}", i.to_string()); panic!() }
        };

        match rows.next() {
            Ok(i) => {
                match i {
                    Some(_) => return true,
                    None => return false
                }
            },
            Err(_) => return false
        }
    }

    fn delete_alert_dao(id: String) -> () {
        let conn = Connection::open("my.sqlite").unwrap();
        let mut stmt = match conn.prepare("DELETE FROM alerts WHERE id = :id") {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };

        let mut rows = match stmt.query_named(&[(":id", &id)]) {
            Ok(i) => i,
            Err(i) => { println!("A {}", i.to_string()); panic!() }
        };

        while let Ok(row) = rows.next() {
            match row {
                Some(i) => (),
                None => break
            }
        }
    }



    // Controller layer
    #[get("/alerts/v1/read")]
    pub fn read_list(token: AdminToken) -> Json<AlertListWrapper> {
        let ret = read_alert_list_dao();
        match ret {
            Ok(i) => Json(i),
            Err(x) => panic!(x)
        }
    }

    #[get("/alerts/v1/read/<id>")]
    pub fn read_get(id: String, token: AdminToken) -> Json<AlertWrapper> {
        let ret = match read_alerts_dao(id) {
            Ok(i) => i,
            Err(i) => { println!("G {}", i.to_string()); panic!() }
        };
        Json(ret)
    }

    #[get("/alerts/v1/write/<id>")]
    pub fn write_get(id: String, token: AlerterToken) -> Json<AlertInstance> {
        let now = Utc::now();

        let a1 = AlertInstance {
            id: id,
            timestamp: now
        };

        write_alert_dao(&a1);
        Json(a1)
    }

    #[get("/alerts/v1/register/<id>")]
    pub fn register_get(id: String, token: AdminToken) -> Json<AlertInstance> {
        let a1 = AlertInstance{ 
            id: id.to_owned(),
            timestamp: Utc.timestamp(0,0)
        };

        if register_find_any(id) == false {
            write_alert_dao(&a1);
        }

        Json(a1)
    }

    #[get("/alerts/v1/delete/<id>")]
    pub fn delete_get(id: String, token: AdminToken) -> () {
        delete_alert_dao(id);
    }


    #[derive(Serialize, Deserialize)]
    pub struct AlertListWrapper {
        alerts: Vec<AlertInstance>,
        version: String
    }

    #[derive(Serialize, Deserialize)]
    pub struct AlertWrapper {
        alert: Option<Alert>,
        version: String
    }

    #[derive(Serialize, Deserialize)]
    struct Alert {
        id: String,
        fired_timestamps: Vec<DateTime<Utc>>
    }

    #[derive(Serialize, Deserialize)]
    pub struct AlertInstance {
        id: String,
        timestamp: DateTime<Utc>
    }

}