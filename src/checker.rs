use oso::ToPolar;
use rocket::{Request, State};

use super::{Error, RocketOso};

/// Policy check builder and verifier
///
/// A `PolicyChecker` actually performs the policy match against oso's policy
/// database. It has access to the HTTP request as well as the configured
/// [`Resolver`](trait.Resolver.html) to retrieve the current actor.
#[async_trait]
pub trait PolicyChecker: Default {
  async fn check<A>(&self, request: &Request<'_>, policies: State<'_, RocketOso<'_, A>>) -> Result<bool, Error>
  where
    A: ToPolar;
}

/// Default `PolicyChecker` for simple route-based policies
///
/// This default implementation for a `PolicyCheck` uses the resolved actor,
/// the request method as the _action_, and the request path as the _resource_.
#[derive(Default)]
pub struct RoutePolicyChecker;

#[async_trait]
impl PolicyChecker for RoutePolicyChecker {
  async fn check<A>(&self, request: &Request<'_>, policies: State<'_, RocketOso<'_, A>>) -> Result<bool, Error>
  where
    A: ToPolar,
  {
    let actor = policies.resolver.resolve_actor(request).await;
    let method = request.method().as_str();
    let path = request.uri().path().to_string();

    let mut oso = policies.oso.lock().unwrap();

    oso.is_allowed(actor, method, path).map_err(|err| Error::OsoError(err))
  }
}
