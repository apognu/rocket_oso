use oso::ToPolar;
use rocket::Request;

/// Dynamic policy actor resolver
///
/// A resolver is used to dynamically resolve the appropriate actor for the
/// current HTTP request. It can return any value that can be used against
/// oso's policy matching (`ToPolar`). This includes most primitive types as
/// well as custom type deriving `PolarClass`.
///
/// # Examples
///
/// ```ignore
/// struct CustomResolver;
///
/// #[async_trait]
/// impl Resolver for CustomResolver {
///   type Actor = String;
///
///   async fn resolve_actor(&self, request: &Request<'_>) -> Self::Actor {
///     request.headers().get_one("X-User").unwrap_or("").to_string()
///   }
/// }
/// ```
#[async_trait]
pub trait Resolver: Send + Sync {
  type Actor: ToPolar;

  async fn resolve_actor(&self, request: &Request<'_>) -> Self::Actor;
}
