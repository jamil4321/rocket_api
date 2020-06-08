#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate lazy_static;
extern crate rocket_cors;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rocket_contrib::json::{Json, JsonValue};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::http::Method; // 1.
use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Error, // 2.
    Cors, CorsOptions // 3.
};
use rocket::State;

use mongodb::{
    
    bson::{self,doc, Bson, document::Document},
    sync::{Client, Collection},
};

 
fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[ // 4.      
        "http://localhost:8000",
        "http://localhost:8080",
        "http://0.0.0.0:8000",        
        // "chrome-extension://fhbjgbiflinjbdggehcddcbncdddomop",               
    ]);

    CorsOptions { // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get,Method::Post].into_iter().map(From::from).collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin", // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn mongo_conection()->Result<Collection,mongodb::error::Error>{
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    let database = client.database("mydb");
    let collection = database.collection("books");
    Ok(collection)
}

#[derive(Debug,FromForm, Clone, Serialize, Deserialize)]
struct Message {
    title: String,
    author:String
}
impl Message{
    fn new(t:String,a:String)->Message{
        Message{
            title:t,
            author:a
        }
    }
}
#[post("/add", data="<user_input>")]
fn mongoPost(user_input: Form<Message>){
    println!("{:?}",user_input);
   let doc = doc! {
       "title": &user_input.title,
       "author": &user_input.author
   };
    match mongo_conection() {
        Ok(col) => col.insert_one(doc.clone(), None),
        Err(e) => Err(e.into()),
    };
    
}
#[get("/get")]
fn mongoGet() ->JsonValue{
    let mut t : String = String::from("");
    let mut a :String  = String::from("");
    let mut book_vec :Vec<Message> = Vec::new();

    match mongo_conection() {
        Ok(coll) => {
            let mut cursor = coll.find(None, None).unwrap();
            for result in cursor{
                if let Ok(item) = result {
                    if let Some(&Bson::String(ref title)) = item.get("title") {
                        t = format!("{}",title);
                    }
                    if let Some(&Bson::String(ref author)) = item.get("author") {
                        a = format!("{}",author);
                    }
                    let book_info = Message::new(t.clone(),a.clone());
                    book_vec.push(book_info)

                }
                
            }
        },
        Err(e) => panic!("Error {:?}",e),
    }
    json!(book_vec)
}

#[get("/")]
fn index()-> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}
fn main() {
    rocket().launch();
}

fn rocket()-> rocket::Rocket{
    rocket::ignite()
    .mount("/", routes![index,mongoPost,mongoGet]).attach(make_cors())
}