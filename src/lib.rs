#[macro_use]
extern crate rocket;
#[macro_use]
extern crate thiserror;

mod checker;
mod guard;
mod resolver;

pub mod prelude {
  pub use crate::{checker::PolicyChecker, guard::Policy, resolver::Resolver, RocketOso};
}

use std::sync::{Arc, Mutex};

use oso::Oso;

pub use self::{
  checker::{PolicyChecker, RoutePolicyChecker},
  guard::Policy,
  resolver::Resolver,
};

#[derive(Debug, Error)]
pub enum Error {
  #[error("policy did not check out, action is forbidden")]
  Unauthorized,
  #[error("no state found for RocketOso")]
  RocketOsoNotFound,
  #[error("oso error: {0}")]
  OsoError(#[from] oso::OsoError),
}

/// Entrypoint to the framework
///
/// This struct is to be provided as a managed state into Rocket and will be
/// used every time a route including the `Policy` guard is called.
///
/// # Fields
///
/// * `oso` - an oso instance with the policy rules loaded
/// * `resolver` - an instance of a [`Resolver`](trait.Resolver.html), used to retrieve actor name from request
///
/// # Examples
///
/// ```ignore
/// let mut oso = Oso::new();
/// oso.load_file("policies.polar");
///
/// rocket::ignite()
///   .manage(RocketOso::new(oso, &MyResolver))
///   .await?;
/// ```
pub struct RocketOso<'p, P> {
  pub oso: Arc<Mutex<Oso>>,
  pub resolver: &'p dyn Resolver<Actor = P>,
}

impl<'p, P> RocketOso<'p, P> {
  pub fn new(oso: Oso, resolver: &'p dyn Resolver<Actor = P>) -> Self {
    Self {
      oso: Arc::new(Mutex::new(oso)),
      resolver,
    }
  }
}
