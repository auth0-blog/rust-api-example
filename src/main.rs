#[macro_use] 
extern crate nickel;
extern crate rustc_serialize;

#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate hyper;
extern crate crypto;
extern crate jwt;

// Nickel
use nickel::{Nickel, JsonBody, HttpRouter, Request, Response, MiddlewareResult, MediaType};
use nickel::status::StatusCode::{self, Forbidden};

// MongoDB
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::error::Result as MongoResult;

// bson
use bson::{Bson, Document};
use bson::oid::ObjectId;

// rustc_serialize
use rustc_serialize::json::{Json, ToJson};
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64};

// hyper
use hyper::header;
use hyper::header::{Authorization, Bearer};
use hyper::method::Method;

// jwt
use std::default::Default;
use crypto::sha2::Sha256;
use jwt::{
    Header,
    Registered,
    Token,
};

#[derive(RustcDecodable, RustcEncodable)]
struct User {
    firstname: String,
    lastname: String,
    email: String
}

static AUTH_SECRET: &'static str = "your_secret_key";

#[derive(RustcDecodable, RustcEncodable)]
struct UserLogin {
    email: String,
    password: String
}

fn get_data_string(result: MongoResult<Document>) -> Result<Json, String> {
    match result {
        Ok(doc) => Ok(Bson::Document(doc).to_json()),
        Err(e) => Err(format!("{}", e))
    }
}

fn authenticator<'mw>(request: &mut Request, response: Response<'mw>, ) -> MiddlewareResult<'mw> {

  // Check if we are getting an OPTIONS request
  if request.origin.method.to_string() == "OPTIONS".to_string() {

      // The middleware shouldn't be used for OPTIONS, so continue
      response.next_middleware()

  } else {

    // We don't want to apply the middleware to the login route
    if request.origin.uri.to_string() == "/login".to_string() {

        response.next_middleware()

    } else {

        // Get the full Authorization header from the incoming request headers
        let auth_header = match request.origin.headers.get::<Authorization<Bearer>>() {
            Some(header) => header,
            None => panic!("No authorization header found")
        };

        // Format the header to only take the value
        let jwt = header::HeaderFormatter(auth_header).to_string();

        // We don't need the Bearer part, 
        // so get whatever is after an index of 7
        let jwt_slice = &jwt[7..];

        // Parse the token
        let token = Token::<Header, Registered>::parse(jwt_slice).unwrap();

        // Get the secret key as bytes
        let secret = AUTH_SECRET.as_bytes();

        // Generic example
        // Verify the token
        if token.verify(&secret, Sha256::new()) {
          
            response.next_middleware()         
          
        } else {

            response.error(Forbidden, "Access denied")

        }

    }
  }
}

fn main() {

    let mut server = Nickel::new();
    let mut router = Nickel::router();

    server.utilize(authenticator);

    router.post("/login", middleware! { |request|

        // Accept a JSON string that corresponds to the User struct
        let user = request.json_as::<UserLogin>().unwrap();

        // Get the email and password
        let email = user.email.to_string();
        let password = user.password.to_string();

        // Simple password checker
        if password == "secret".to_string() {

            let header: Header = Default::default();

            // For the example, we just have one claim
            // You would also want iss, exp, iat etc
            let claims = Registered {
                sub: Some(email.into()),
                ..Default::default()
            };

            let token = Token::new(header, claims);

            // Sign the token
            let jwt = token.signed(AUTH_SECRET.as_bytes(), Sha256::new()).unwrap();

            format!("{}", jwt)

        } else {
            format!("Incorrect username or password")
        }

    });

    router.get("/users", middleware! { |request, mut response|

        // Connect to the database
        let client = Client::connect("localhost", 27017)
          .ok().expect("Error establishing connection.");

        // The users collection
        let coll = client.db("rust-users").collection("users");

        // Create cursor that finds all documents
        let cursor = coll.find(None, None).unwrap();

        // Opening for the JSON string to be returned
        let mut data_result = "{\"data\":[".to_owned();

        for (i, result) in cursor.enumerate() {
            match get_data_string(result) {
                Ok(data) => {
                    let string_data = if i == 0 { 
                        format!("{}", data)
                    } else {
                        format!("{},", data)
                    };

                    data_result.push_str(&string_data);
                },

                Err(e) => return response.send(format!("{}", e))
            }
        }

        // Close the JSON string
        data_result.push_str("]}");

        // Set the returned type as JSON
        response.set(MediaType::Json);

        // Send back the result
        format!("{}", data_result)

    });

    router.post("/users/new", middleware! { |request, response|

        // Accept a JSON string that corresponds to the User struct
        let user = request.json_as::<User>().unwrap();

        let firstname = user.firstname.to_string();
        let lastname = user.lastname.to_string();
        let email = user.email.to_string();

        // Connect to the database
        let client = Client::connect("localhost", 27017)
            .ok().expect("Error establishing connection.");

        // The users collection
        let coll = client.db("rust-users").collection("users");

        // Insert one user
        match coll.insert_one(doc! { 
            "firstname" => firstname,
            "lastname" => lastname,
            "email" => email 
        }, None) {
            Ok(_) => (StatusCode::Ok, "Item saved!"),
            Err(e) => return response.send(format!("{}", e))
        }

    });

    router.delete("/users/:id", middleware! { |request, response|

        let client = Client::connect("localhost", 27017)
            .ok().expect("Failed to initialize standalone client.");

        // The users collection
        let coll = client.db("rust-users").collection("users");

        // Get the user_id from the request params
        let object_id = request.param("id").unwrap();

        // Match the user id to an bson ObjectId
        let id = match ObjectId::with_string(object_id) {
            Ok(oid) => oid,
            Err(e) => return response.send(format!("{}", e))
        };

        match coll.delete_one(doc! {"_id" => id}, None) {
            Ok(_) => (StatusCode::Ok, "Item deleted!"),
            Err(e) => return response.send(format!("{}", e))
        }

    });

    server.utilize(router);

    server.listen("127.0.0.1:9000");
}