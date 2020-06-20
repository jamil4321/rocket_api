#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
extern crate rocket_cors;
use rocket_contrib::json::{Json, JsonValue};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::http::Method; 
use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Error,
    Cors, CorsOptions 
};
use rand::random;
use mongodb::{
    
    bson::{self,doc, Bson, document::Document},
    sync::{Client, Collection},
    results::InsertOneResult
};

const Book:&str = "book";
 
fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[     
        "http://localhost:3000",
        "http://localhost:8080",
        "http://0.0.0.0:8000",
        "http://0.0.0.0:3000",                
    ]);

    CorsOptions { 
        allowed_origins,
        allowed_methods: vec![Method::Get,Method::Post,Method::Delete,Method::Put].into_iter().map(From::from).collect(), 
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Headers", 
            "Access-Control-*",
            "Origin", 
            "X-Requested-With", 
            "Content-Type", 
            "Accept"
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}


fn mongo_conection(collection:&str)->Result<Collection,mongodb::error::Error>{
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    let database = client.database("mydb");
    let collection = database.collection(collection);
    Ok(collection)
}

#[derive(Debug,FromForm, Clone, Serialize, Deserialize)]
struct Message {
    id:i64,
    title: String,
    author:String
}
impl Message{
    fn new(id:i64,t:String,a:String)->Message{
        Message{
            id:id,
            title:t,
            author:a
        }
    }
}
#[post("/",format="application/json", data="<user_input>")]
fn mongoPost(user_input: Json<Message>)->Result<Json<Message>,mongodb::error::Error>{
    println!("{:?}",user_input);

   let doc = doc!{
       "id":&user_input.id,
       "title": &user_input.title,
       "author": &user_input.author
   };
    match mongo_conection(&Book) {
        Ok(col) => {
            col.insert_one(doc.clone(), None);
            Ok(user_input)
        }
        Err(e) => Err(e.into()),
    }
    
}
#[get("/")]
fn mongoGet() ->JsonValue{
    let mut i : i64 = 0;
    let mut t : String = String::from("");
    let mut a :String  = String::from("");
    let mut book_vec :Vec<Message> = Vec::new();

    match mongo_conection(&Book) {
        Ok(coll) => {
            let mut cursor = coll.find(None, None).unwrap();
            for result in cursor{
                if let Ok(item) = result {
                    if let Some(&Bson::Int64(id)) = item.get("id") {
                        i = id
                    }
                    if let Some(&Bson::String(ref title)) = item.get("title") {
                        t = format!("{}",title);
                    }
                    if let Some(&Bson::String(ref author)) = item.get("author") {
                        a = format!("{}",author);
                    }
                    let book_info = Message::new(i,t.clone(),a.clone());
                    book_vec.push(book_info)

                }
                
            }
        },
        Err(e) => panic!("Error {:?}",e),
    }
    json!(book_vec)
}

#[delete("/<id>")]
fn mongoDelete(id:i64)->Result<String,mongodb::error::Error>{
    println!("{}",id);
    match mongo_conection(&Book) {
        Ok(col) => {
            col.delete_one(doc!("id":id), None);
            let string = format!("deleted {}",id);
            Ok(string)
        },
        Err(e) => Err(e.into()),
    }
    
}


#[put("/<id>",format="application/json", data ="<user_input>")]
fn mongoPut(id:i64,user_input:Json<Message>) ->Result<Json<Message>,mongodb::error::Error> {
        let new_data = doc!{
            "id":&user_input.id,
            "title": &user_input.title,
            "author": &user_input.author
        };
    match mongo_conection(&Book) {
        Ok(col) => {
            col.replace_one(doc!("id":id),new_data ,None);
            Ok(user_input)
        },
        Err(e) => Err(e.into()),
    }
}

fn main() {
    rocket().launch();
}

fn rocket()-> rocket::Rocket{
    rocket::ignite()
    .mount("/", routes![mongoPost,mongoGet,mongoDelete,mongoPut]).attach(make_cors())
}