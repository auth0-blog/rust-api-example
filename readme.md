# Rust API Example

This repo shows how to implement a RESTful API in Rust with **[Nickel.rs](http://nickel.rs/)** and the **[MongoDB Rust Driver](https://github.com/mongodb-labs/mongo-rust-driver-prototype)**.

## Important Snippets

The simple example has `GET`, `POST`, and `DELETE` routes in the `main` function.

The **GET** `/users` route searches MongoDB for all users and then returns a JSON string of the data.

```rust
// src/main.rs

...

fn get_data_string(result: MongoResult<Document>) -> Result<Json, String> {
    match result {
        Ok(doc) => Ok(Bson::Document(doc).to_json()),
        Err(e) => Err(format!("{}", e))
    }
}

router.get("/users", middleware! { |request, response|

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

    // Send back the result
    format!("{}", data_result)

});

...
```

The **POST** `/users/new` route takes JSON data and saves in the database. The data conforms to the `User` struct.

```rust
// src/main.rs

...

#[derive(RustcDecodable, RustcEncodable)]
struct User {
    firstname: String,
    lastname: String,
    email: String
}

...

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

...
```

The **DELETE** `/users/:user_id` takes an `objectid` as a parameter, decodes it into BSON, and deletes it from the database.

```rust
// src/main.rs

...

router.delete("/users/:user_id", middleware! { |request, response|

    let client = Client::connect("localhost", 27017)
        .ok().expect("Failed to initialize standalone client.");

    // The users collection
    let coll = client.db("rust-users").collection("users");

    // Get the user_id from the request params
    let user_id = request.param("user_id").unwrap();

    // Match the user id to an bson ObjectId
    let id = match ObjectId::with_string(user_id) {
        Ok(oid) => oid,
        Err(e) => return response.send(format!("{}", e))
    };

    match coll.delete_one(doc! {"_id" => id}, None) {
        Ok(_) => (StatusCode::Ok, "Item deleted!"),
        Err(e) => return response.send(format!("{}", e))
    }

});

...
```

## What is Auth0?

Auth0 helps you to:

* Add authentication with [multiple authentication sources](https://docs.auth0.com/identityproviders), either social like **Google, Facebook, Microsoft Account, LinkedIn, GitHub, Twitter, Box, Salesforce, amont others**, or enterprise identity systems like **Windows Azure AD, Google Apps, Active Directory, ADFS or any SAML Identity Provider**.
* Add authentication through more traditional **[username/password databases](https://docs.auth0.com/mysql-connection-tutorial)**.
* Add support for **[linking different user accounts](https://docs.auth0.com/link-accounts)** with the same user.
* Support for generating signed [Json Web Tokens](https://docs.auth0.com/jwt) to call your APIs and **flow the user identity** securely.
* Analytics of how, when and where users are logging in.
* Pull data from other sources and add it to the user profile, through [JavaScript rules](https://docs.auth0.com/rules).

## Create a free account in Auth0

1. Go to [Auth0](https://auth0.com) and click Sign Up.
2. Use Google, GitHub or Microsoft Account to login.

## Author

[Auth0](https://auth0.com)