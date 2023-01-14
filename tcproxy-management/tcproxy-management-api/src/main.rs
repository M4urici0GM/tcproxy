use std::env;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder};
use actix_web::web::{Data, Json};
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{Bson, doc};
use serde::{Deserialize, Serialize};
use tcproxy_core::Error;

#[derive(Serialize)]
struct HelloWorld {
    message: String
}

#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    username: String,
    name: String,
}

#[derive(Clone)]
struct UserRepository {
    collection: Collection<User>
}

trait IntoObjectId {
    fn into_object_id(&self) -> tcproxy_core::Result<ObjectId>;
}

impl IntoObjectId for Bson {
    fn into_object_id(&self) -> tcproxy_core::Result<ObjectId> {
        if let Bson::ObjectId(id) = self {
            return Ok(id.to_owned())
        }

        Err("error trying to convert to objectid".into())
    }
}

impl UserRepository {
    pub fn new(mongodb: &Database) -> Self {
        Self {
            collection: mongodb.collection("users")
        }
    }

    pub async fn new_user(&self, user: User) -> tcproxy_core::Result<ObjectId> {
        let created_user = match self.collection
            .insert_one(&user, None)
            .await {
            Ok(result) => result,
            Err(err) => return Err(err.into())
        };

        Ok(created_user.inserted_id.into_object_id()?)
    }
}

#[post("/")]
async fn create_user(user_repo: Data<UserRepository>, user: Json<User>) -> impl Responder {
    let user = User {
        id: None,
        username: user.username.to_owned(),
        name: user.name.to_owned(),
    };

    match user_repo.new_user(user).await {
        Ok(user_id) => HttpResponse::Created().json(doc! { "user_id": user_id.to_hex() }),
        Err(aaaa) => {
            println!("{:?}", aaaa);

            HttpResponse::InternalServerError().json(doc!{ "message": String::from("failed to create user") })
        }
    }
}

#[get("/")]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().json(HelloWorld { message: String::from("hello world") })
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();

    let uri = match env::var("MONGOURI") {
        Ok(v) => v.to_string(),
        Err(_) => format!("Error loading env variable"),
    };

    println!("aaaaaaa {uri}");

    let client = Client::with_uri_str(uri).await.unwrap();
    let db = client.database("tcproxy");
    let user_repo = UserRepository::new(&db);

    let app_data = Data::new(user_repo);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(hello_world)
            .service(create_user)
    })
    .bind(("127.0.0.1", 3333))?
    .run()
    .await
}
