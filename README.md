# rocket_oso

`rocket_oso` is a simple framework and request guard for [Rocket](https://github.com/SergioBenitez/Rocket) to authorize incoming HTTP requests against [oso](https://github.com/osohq/oso)'s policy definitions.

It provides a request guard that will check the given actor is allowed to access the current route, and short-circuit the request into a `401 Unauthorized` if it is not.

It does not replace finer grained authorization that could implement your business logic.

This follows Rocket's repository tip, so it can run async and on stable Rust. This will prevent it from being published on [crates.io](https://crates.io) until Rocket 0.5 is out.

## Usage

### Setting up

`rocket_oso` needs two things to work: a copy of `Oso` with policies loaded, and a way to retrieve the actor for a request.

First, define the way we can resolve the actor from the request object by implementing `Resolver`:

```rust
use rocket_oso::prelude::*;

struct StaticResolver;

#[async_trait]
impl Resolver for StaticResolver {
  type Actor = String;

  async fn resolve_actor(&self, request: &Request<'_>) -> Self::Actor {
    "apognu@example.com".to_string()
  }
}
```

Then, register `RocketOso` as a managed state into Rocket:

```rust
let mut oso = Oso::new();
oso.load_file("policies.polar")?;

rocket::ignite()
  .manage(RocketOso::new(oso, &StaticResolver))
  [...]
  .launch()
  .await?;
```

### Guarding routes

`rocket_oso` provides a request guard, `Policy<A>`, that, by default, will look up a policy matching the actor resolved by the `Resolver`, the request method and path in order to make a decision on whether the request should proceed or be stopped.

```rust
#[get("/private/hello")]
fn hello(_policy: Policy<String>) -> &'static str {
  "Hello from the private area!"
}
```

The type parameter in `Policy<A>` must match the actor type returned by your registered resolver. Here, this is a string. In more complex system (where the actor is resolved through a database, for instance), this type could be a complex struct.

## Customization

On top of the `Resolver`, whcih allows you to dynamically fetch the current actor, you can fine-tune exactly what information is used to look up oso's policy (by default, the HTTP method and path are used as oso's _action_ and _resource_). Here is how you can define and use your own `PolicyChecker`:

```rust
#[derive(Default)]
struct CustomPolicyChecker;

#[async_trait]
impl PolicyChecker for CustomPolicyChecker {
  async fn apply<A>(&self, request: &Request<'_>, policies: State<'_, RocketOso<'_, A>>) -> Result<bool, Error>
  where
    A: ToPolar,
  {
    let actor = policies.resolver.resolve_actor(request).await;
    let action = "CUSTOM ACTION";
    let resource = "CUSTOM RESOURCE";

    let mut oso = policies.oso.lock().unwrap();

    oso.is_allowed(actor, action, resource).map_err(|err| Error::OsoError(err))
  }
}
```

Having access to the full HTTP request, you could build up some nice intricate authorization policies here. To use this checker in a request guard, you can use the second type parameter to `Policy`:

```rust
#[get("/hello")]
fn hello(_policy: Policy<User, CustomPolicyChecker>) -> &'static str {
  "Hello, world!"
}
```
