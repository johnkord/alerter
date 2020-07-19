pub mod alerts {
    use std::collections::HashMap;
    use std::cmp::max;
    use serde::{Deserialize, Serialize};
    use rocket_contrib::json::Json;
    use chrono::{DateTime, TimeZone, Utc};
    
    use rusqlite::{params, named_params, Connection, Result};

    use crate::tokens::admin_tokens;
    use admin_tokens::admin_tokens::*;
    use crate::tokens::alerter_tokens;
    use alerter_tokens::alerter_tokens::*;
    use log::{info, warn, error};

    use rocket::response::status::*;
    use rocket::http::Status;
    use std::cmp::Ordering;

    enum AlertError {
        ConnectionToDBFailed,
        PrepareFailed,
        QueryFailed,
        RowError,
    }

    struct AlertListTableEntry {
        timestamp: Option<DateTime<Utc>>,
        awaiting: Option<bool>
    }

    fn get_max_timestamp(x: &Option<DateTime<Utc>>, y: &Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
        if *x == None {
            if *y == None {
                return None;
            } else {
                return y.to_owned();
            }
        } else if *y == None {
            return x.to_owned();
        } else {
            return Some(Utc.timestamp(max(x.unwrap().timestamp(), y.unwrap().timestamp()), 0));
        }
    }
    
    fn read_alert_list_dao() -> Result<AlertListWrapper, AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        
        
        let mut stmt = match conn.prepare("SELECT id, awaiting FROM alerts_registry") {
            Ok(x) => x,
            Err(err) => { error!("Error preparing statement: {}", err); return Err(AlertError::PrepareFailed) }
        };

        let all_registered_alerts: Vec<AlertInstance> = match stmt.query_map(params![], |row| {
            let id: String = match row.get(0) {
                Ok(x) => x,
                Err(err) => { error!("Error converting to String: {}", err); return Err(err) }
            };
            let awaiting: Option<bool> = match row.get(1) {
                Ok(x) => Some(x),
                Err(err) => { None }
            };

            Ok(AlertInstance { id: id.to_owned(), timestamp: None, awaiting: awaiting })
        }) {
            Ok(x) => x.filter_map(|y| match y {
                Ok(y) => Some(y),
                Err(err) => None
            }).collect(),
            Err(err) =>  { error!("Error performing query: {}", err); return Err(AlertError::QueryFailed) }
        };

        let mut unawaiting_v: Vec<AlertInstance> = Vec::new();
        let mut awaiting_v: Vec<AlertInstance> = Vec::new();

        all_registered_alerts.iter().for_each(|x| match x.awaiting {
            None => unawaiting_v.push( AlertInstance{ id: x.id.to_owned(), awaiting: x.awaiting.to_owned(), timestamp: None } ),
            Some(y) => match y {
                true => awaiting_v.push( AlertInstance{ id: x.id.to_owned(), awaiting: x.awaiting.to_owned(), timestamp: None } ),
                false => unawaiting_v.push( AlertInstance{ id: x.id.to_owned(), awaiting: x.awaiting.to_owned(), timestamp: None } )
            }
        });
        
        let mut most_recent_alerts_unawaiting: HashMap<String, AlertListTableEntry> = HashMap::new();
        let mut most_recent_alerts_awaiting: HashMap<String, AlertListTableEntry> = HashMap::new();
        
        for instance in unawaiting_v {
            most_recent_alerts_unawaiting.insert(instance.id, AlertListTableEntry{ timestamp: None, awaiting: instance.awaiting });
        }

        for instance in awaiting_v {
            most_recent_alerts_awaiting.insert(instance.id, AlertListTableEntry{ timestamp: None, awaiting: instance.awaiting });
        }

        let mut stmt = match conn.prepare("SELECT alerts.id, alerts.timestamp, alerts_registry.awaiting FROM alerts LEFT JOIN alerts_registry ON alerts.id = alerts_registry.id") {
            Ok(x) => x,
            Err(err) => { error!("Error preparing statement: {}", err); return Err(AlertError::PrepareFailed) }
        };

        let alert_instances = match stmt.query_map(params![], |row| {
            let ts: i64 = match row.get(1) {
                Ok(x) => x,
                Err(err) => { println!("B {}", err.to_string()); panic!() }
            };
            Ok(AlertInstance {
                id: match row.get(0) {
                    Ok(x) => x,
                    Err(err) => { println!("C {}", err.to_string()); panic!() }
                },
                timestamp: Some(Utc.timestamp(ts, 0)),
                awaiting: match row.get(2) {
                    Ok(x) => { println!("C1 {}", x); Some(x) },
                    Err(err) => { println!("C2 {}", err.to_string()); None }
                },
            })
        }) {
           Ok(x) => x,
           Err(err) =>  { error!("Error performing query: {}", err); return Err(AlertError::QueryFailed) }
        };

        for alert_instance in alert_instances {
            match alert_instance {
                Ok(x) => {
                    if most_recent_alerts_awaiting.contains_key(&x.id) {
                        match most_recent_alerts_awaiting.get(&x.id) {
                            Some(y) => {
                                let more_recent_timestamp: Option<DateTime<Utc>> = get_max_timestamp(&x.timestamp, &y.timestamp);
                                
                                most_recent_alerts_awaiting.insert(x.id, AlertListTableEntry { 
                                    timestamp: more_recent_timestamp,
                                    awaiting: y.awaiting
                                });
                            },
                            None => {
                                most_recent_alerts_awaiting.insert(x.id, AlertListTableEntry { 
                                    timestamp: x.timestamp,
                                    awaiting: Some(false)
                                });
                            }
                        };
                    } else {
                        match most_recent_alerts_unawaiting.get(&x.id) {
                            Some(y) => {
                                let more_recent_timestamp: Option<DateTime<Utc>> = get_max_timestamp(&x.timestamp, &y.timestamp);
                                
                                most_recent_alerts_unawaiting.insert(x.id, AlertListTableEntry { 
                                    timestamp: more_recent_timestamp,
                                    awaiting: y.awaiting
                                });
                            },
                            None => {
                                most_recent_alerts_unawaiting.insert(x.id, AlertListTableEntry { 
                                    timestamp: x.timestamp,
                                    awaiting: Some(false)
                                });
                            }
                        }
                    }
                },
                Err(err) =>  { warn!("Row in returned alert_instances had an error: {}", err); return Err(AlertError::RowError) }
            };
        }

        let mut flattened_awaiting_alerts = most_recent_alerts_awaiting.iter()
            .map(|x| AlertInstance{ 
                id: x.0.to_owned(),
                timestamp: x.1.timestamp.to_owned(),
                awaiting: x.1.awaiting
            })
            .collect::<Vec<AlertInstance>>();

        let mut flattened_unawaiting_alerts = most_recent_alerts_unawaiting.iter()
            .map(|x| AlertInstance{ 
                id: x.0.to_owned(),
                timestamp: x.1.timestamp.to_owned(),
                awaiting: x.1.awaiting
            })
            .collect::<Vec<AlertInstance>>();

        flattened_awaiting_alerts.sort_by(|x,y| {
            if x.timestamp == None {
                if y.timestamp == None {
                    return Ordering::Equal;
                } else {
                    return Ordering::Greater;
                }
            } else if y.timestamp == None {
                return Ordering::Less;
            } else {
                return (y.timestamp.unwrap() - x.timestamp.unwrap()).cmp(&chrono::Duration::seconds(0));
            }
        });

        flattened_unawaiting_alerts.sort_by(|x,y| {
            if x.timestamp == None {
                if y.timestamp == None {
                    return Ordering::Equal;
                } else {
                    return Ordering::Greater;
                }
            } else if y.timestamp == None {
                return Ordering::Less;
            } else {
                return (y.timestamp.unwrap() - x.timestamp.unwrap()).cmp(&chrono::Duration::seconds(0));
            }
        });

        flattened_unawaiting_alerts.iter().for_each(|x| { 
            flattened_awaiting_alerts.push( AlertInstance {
                id: x.id.to_owned(),
                timestamp: x.timestamp,
                awaiting: x.awaiting
            })
        });

        let list = AlertListWrapper{ 
            alerts: flattened_awaiting_alerts,
            version: "1.0.0".to_owned()
        };

        info!("Returning an AlertListWrapper with {} number of AlertInstances in it", list.alerts.len());

        Ok(list)
    }

    fn read_alerts_dao(id: String) -> Result<AlertWrapper, AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        let mut stmt = match conn.prepare("SELECT alerts.id, alerts.timestamp, alerts_registry.awaiting FROM alerts LEFT JOIN alerts_registry ON alerts.id = alerts_registry.id WHERE alerts.id = :id") {
            Ok(x) => x,
            Err(err) => { error!("Error preparing statement: {}", err); return Err(AlertError::PrepareFailed) }
        };

        let mut v: Vec<AlertInstance> = match stmt.query_map(params![id], |row| {
            let ts = match row.get(1) {
                Ok(x) => Some(Utc.timestamp(x, 0)),
                Err(err) => { error!("Error converting to timestamp: {}", err); return Err(err) }
            };
            let awaiting: Option<bool> = match row.get(2) {
                Ok(x) => Some(x),
                Err(err) => { None }
            };

            Ok(AlertInstance { id: id.to_owned(), timestamp: ts, awaiting: awaiting })
        }) {
            Ok(x) => x.filter_map(|y| match y {
                Ok(y) => Some(y),
                Err(err) => None
            }).collect(),
            Err(err) =>  { error!("Error performing query: {}", err); return Err(AlertError::QueryFailed) }
        }; 

        v.sort_by(|x,y| { 
            ((*y).timestamp.unwrap() - (*x).timestamp.unwrap()).cmp(&chrono::Duration::seconds(0))
        });

        info!("Returning an AlertWrapper with {} number of fired_timestamps in it", v.len());

        let awaiting_value = match v.len() {
            0 => None,
            _ => v[0].awaiting
        };

        let a1 = Alert{
            id: id,
            fired_timestamps: v.iter().map(|x| return x.timestamp.unwrap()).collect(),
            awaiting: awaiting_value
        };

        Ok(AlertWrapper { alert: Some(a1), version: "1.0.0".to_owned() })
    }

    fn write_alert_dao(a: &AlertInstance) -> Result<&AlertInstance, AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        match conn.execute(
            "INSERT INTO alerts (id, timestamp)
                    VALUES (?1, ?2)",
            params![a.id, a.timestamp.unwrap().timestamp()],
        ) {
            Ok(_) => Ok(a),
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::QueryFailed) }
        }

    }

    fn register_alert_dao(id: String) -> Result<bool, AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        let default_awaiting = true;

        match conn.execute(
            "INSERT INTO alerts_registry (id, awaiting)
                    VALUES (?1, ?2)",
            params![id, default_awaiting],
        ) {
            Ok(_) => return Ok(default_awaiting),
            Err(err) => { error!("Error performing query: {}", err); return Ok(true) }
        };
    }

    fn set_awaiting_dao(id: String, requested_state: bool) -> Result<bool, AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        match register_alert_dao(id.to_owned()) {
            Ok(_) => (),
            Err(err) => { error!("Error registering alert to DB"); return Err(AlertError::ConnectionToDBFailed) }
        }

        match conn.execute("UPDATE alerts_registry SET awaiting = ?1 WHERE id = ?2", params![requested_state, id]) {
            Ok(_) => Ok(requested_state),
            Err(err) => { error!("Error performing query: {}", err); Err(AlertError::QueryFailed) }
        }
    }

    fn delete_alert_dao(id: String) -> Result<(),AlertError> {
        let conn = match Connection::open("my.sqlite") {
            Ok(x) => x,
            Err(err) => { error!("Error connecting to DB: {}", err); return Err(AlertError::ConnectionToDBFailed) }
        };

        match conn.execute("DELETE FROM alerts WHERE id = :id", &[&id]) {
            Ok(x) => (),
            Err(err) => { error!("Error performing query: {}", err); return Err(AlertError::QueryFailed) }
        };

        match conn.execute("DELETE FROM alerts_registry WHERE id = :id", &[&id]) {
            Ok(x) => (),
            Err(err) => { error!("Error performing query: {}", err); return Err(AlertError::QueryFailed) }
        };

        Ok(())
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
            Err(i) => { panic!() }
        };
        Json(ret)
    }

    #[get("/alerts/v1/write/<id>")]
    pub fn write_get(id: String, token: AlerterToken) -> Result<Json<AlertInstance>, Status> {
        let now = Utc::now();

        let a1 = AlertInstance {
            id: id.to_owned(),
            timestamp: Some(now),
            awaiting: None
        };

        match set_awaiting_dao(id.to_owned(), false) {
            Ok(x) => (),
            Err(err) => return Err(Status::new(500, "Internal error"))
        }

        match write_alert_dao(&a1) {
            Ok(x) => Ok(Json(a1)),
            Err(err) => Err(Status::new(500, "Internal error"))
        }
    }

    #[get("/alerts/v1/set_awaiting/<id>")]
    pub fn set_awaiting_get(id: String, token: AdminToken) ->  Result<Json<AlertInstance>, Status> {
        match set_awaiting_dao(id.to_owned(), true) {
            Ok(x) => return Ok(Json(AlertInstance{ 
                id: id,
                timestamp: Some(Utc.timestamp(0,0)),
                awaiting: Some(x)
            })),
            Err(err) => return Err(Status::new(500, "Internal error"))
        }
    }

    #[get("/alerts/v1/unset_awaiting/<id>")]
    pub fn unset_awaiting_get(id: String, token: AdminToken) ->  Result<Json<AlertInstance>, Status> {
        match set_awaiting_dao(id.to_owned(), false) {
            Ok(x) => return Ok(Json(AlertInstance{ 
                id: id,
                timestamp: Some(Utc.timestamp(0,0)),
                awaiting: Some(x)
            })),
            Err(err) => return Err(Status::new(500, "Internal error"))
        }
    }
    
    #[get("/alerts/v1/delete/<id>")]
    pub fn delete_get(id: String, token: AdminToken) -> Result<(), Status> {
        match delete_alert_dao(id) {
            Ok(_) => Ok(()),
            Err(err) => Err(Status::new(500, "Internal error"))
        }
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
        fired_timestamps: Vec<DateTime<Utc>>,
        awaiting: Option<bool>
    }

    #[derive(Serialize, Deserialize)]
    pub struct AlertInstance {
        id: String,
        timestamp: Option<DateTime<Utc>>,
        awaiting: Option<bool>
    }
}
