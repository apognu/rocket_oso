#[macro_use]
extern crate rocket;
#[macro_use]
extern crate oso;

use std::error::Error as StdError;

use oso::{Oso, ToPolar};
use rocket::{request::Request, State};

use rocket_oso::{prelude::*, Error};

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

#[derive(Default)]
struct LocalhostPolicyChecker;

#[derive(PolarClass)]
struct CustomAction {
  #[polar(attribute)]
  method: String,
  #[polar(attribute)]
  ip: String,
}

#[async_trait]
impl PolicyChecker for LocalhostPolicyChecker {
  async fn check<A>(&self, request: &Request<'_>, policies: State<'_, RocketOso<'_, A>>) -> Result<bool, Error>
  where
    A: ToPolar,
  {
    let actor = policies.resolver.resolve_actor(request).await;
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let ip = request
      .client_ip()
      .and_then(|ip| Some(ip.to_string()))
      .unwrap_or("".into());

    let action = CustomAction { method, ip };
    let mut oso = policies.oso.lock().unwrap();

    oso.is_allowed(actor, action, path).map_err(|err| Error::OsoError(err))
  }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn StdError>> {
  let mut oso = Oso::new();
  oso.load_file("./examples/policies.polar").unwrap();

  rocket::ignite()
    .manage(RocketOso::new(oso, &CustomActorResolver))
    .mount("/", routes![hello])
    .launch()
    .await
    .unwrap();

  Ok(())
}

#[get("/content")]
fn hello(_policy: Policy<User, LocalhostPolicyChecker>) -> &'static str {
  "Hello, world!"
}
