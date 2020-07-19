#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;


use rusqlite::{Connection};


mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

mod alerts;
mod tokens;
use alerts::alerts::alerts::*;
use tokens::admin_tokens::admin_tokens::*;
use tokens::alerter_tokens::alerter_tokens::*;
use rocket_contrib::serve::StaticFiles;


fn main() -> () {
    // Init logging
    /*
    Builder::new()
    .format(|buf, record| {
        writeln!(buf,
            "{} [{}] - {}",
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            record.level(),
            record.args()
        )
    })
    .filter(None, LevelFilter::Info)
    .init();
    */

    // Init tables and migrations
    let mut conn = Connection::open("my.sqlite").unwrap();
    match embedded::migrations::runner().run(&mut conn) {
        Ok(i) => println!("{}","Successful migration!"),
        Err(i) =>  println!("{} - {}","Error on migration!", i),
    };

    rocket::ignite().mount("/", routes![
                read_list, read_get, write_get, set_awaiting_get, unset_awaiting_get, delete_get,
                list_admin_token_get, write_admin_token_get, delete_admin_token_get,
                list_alerter_token_get, write_alerter_token_get, delete_alerter_token_get])
                .mount("/", StaticFiles::from("web"))
        //.manage(RwLock::new(v))
        .launch();
}
