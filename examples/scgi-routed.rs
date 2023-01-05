use gemfra::{
    error::AnyError, protocol::Scgi, request::Request, response::Response, routed::RoutedApp,
};
use gemfra_codegen::route;

#[route("/")]
async fn main_route(request: Request) -> Result<Response, AnyError> {
    let script = &request.script;
    Ok(Response::success(
        "text/gemini",
        format!(
            "# Hello World

Here is my example routed capsule!

=> {script}/people View your personal page
=> {script}/info/hello View some information on your request
"
        ),
    ))
}

#[route("/people/:name")]
async fn person_route(_request: Request, name: &str) -> Result<Response, AnyError> {
    Ok(Response::success(
        "text/gemini",
        format!(
            "# {name} - A custom page for you

This page was curated specifically for you \"{name}\", Congrats!
",
        ),
    ))
}

#[route("/people")]
async fn person_route_select(request: Request) -> Result<Response, AnyError> {
    if let Some(query) = request.query {
        Ok(Response::redirect(format!(
            "{}{}/{}",
            request.script, request.path, query
        )))
    } else {
        Ok(Response::input("Enter your name"))
    }
}

#[route("/info/*")]
async fn info_route(request: Request) -> Result<Response, AnyError> {
    let url = request.url;
    let addr = request.remote_addr;
    let host = request.remote_host;
    let path = request.path;
    let query = request.query.unwrap_or("".to_owned());

    Ok(Response::success(
        "text/gemini",
        format!(
            "# Info on your request

Your IP is {addr}, with a host of {host}.

Your requested url is '{url}'.

Relative to this cgi, you requested for '{path}'.

Your query is '{query}'
",
        ),
    ))
}

#[tokio::main]
async fn main() {
    let mut app = RoutedApp::new();

    app.register(&main_route);
    app.register(&person_route_select);
    app.register(&person_route);
    app.register(&info_route);

    app.run_scgi("127.0.0.1:8000").await.unwrap();
}
