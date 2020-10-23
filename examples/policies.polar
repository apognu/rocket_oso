allow(actor: User, "GET", "/hello") if
  actor.email == "apognu@example.com";

allow(_, "GET", "/guest");

allow(actor: User, action: CustomAction, "/content") if
  actor.email == "apognu@example.com"
  and action.method == "POST"
  and action.body_size < 10;
