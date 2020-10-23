#[macro_use]
extern crate rocket;
#[macro_use]
extern crate oso;

use std::error::Error;

use oso::Oso;
use rocket::request::Request;

use rocket_oso::prelude::*;

#[derive(PolarClass)]
struct User {
  #[polar(attribute)]
  email: String,
}

struct CustomActorResolver;

#[async_trait]
impl Resolver for CustomActorResolver {
  type Actor = User;

  async fn resolve_actor(&self, request: &Request<'_>) -> Self::Actor {
    let email = request.headers().get_one("X-User").unwrap_or("").to_string();

    User { email }
  }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let mut oso = Oso::new();
  oso.load_file("./examples/policies.polar").unwrap();

  rocket::ignite()
    .manage(RocketOso::new(oso, &CustomActorResolver))
    .mount("/", routes![hello, guest])
    .launch()
    .await
    .unwrap();

  Ok(())
}

#[get("/hello")]
fn hello(_policy: Policy<User>) -> &'static str {
  "Hello, world!"
}

#[get("/guest")]
fn guest(_policy: Policy<User>) -> &'static str {
  "Guest area"
}
