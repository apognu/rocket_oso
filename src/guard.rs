use std::marker::PhantomData;

use oso::ToPolar;
use rocket::{
  http::Status,
  outcome::Outcome,
  request::{self, FromRequest, Request},
  State,
};

use super::{
  checker::{PolicyChecker, RoutePolicyChecker},
  Error, RocketOso,
};

/// Request guard using oso's policy database
///
/// This request guard, when used in a route within Rocket, will perform policy
/// verification against the current request with the resolved actor.
///
/// # Type parameters
///
///  * `A` - type of the resolved actor
///  * `C` - `PolicyChecker` to use, defaults to [`RoutePolicyChecker`](struct.RoutePolicyChecker.html)
pub struct Policy<A, C = RoutePolicyChecker>
where
  A: ToPolar,
{
  _marker: (PhantomData<A>, PhantomData<C>),
}

impl<A, C> Default for Policy<A, C>
where
  A: ToPolar,
{
  fn default() -> Self {
    Self {
      _marker: Default::default(),
    }
  }
}

#[async_trait]
impl<'a, 'r, A, C> FromRequest<'a, 'r> for Policy<A, C>
where
  A: ToPolar,
  C: PolicyChecker + Sync + Send,
{
  type Error = Error;

  async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    if let Outcome::Success(policies) = request.guard::<State<RocketOso<A>>>().await {
      let checker = C::default();

      return match checker.check(request, policies).await {
        Ok(true) => Outcome::Success(Policy::default()),
        Ok(false) => Outcome::Failure((Status::Unauthorized, Error::Unauthorized)),
        Err(err) => Outcome::Failure((Status::InternalServerError, err)),
      };
    }

    Outcome::Failure((Status::InternalServerError, Error::RocketOsoNotFound))
  }
}
