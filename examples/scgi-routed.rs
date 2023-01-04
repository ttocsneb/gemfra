use async_trait::async_trait;
use gemfra::{
    error::{AnyError, ToGemError},
    protocol::Scgi,
    request::Request,
    response::Response,
    routed::{Route, RoutedApp},
};
use route_recognizer::Params;

struct MainRoute;
#[async_trait]
impl Route for MainRoute {
    fn endpoint(&self) -> &str {
        "/"
    }

    async fn handle(&self, _params: &Params, request: Request) -> Result<Response, AnyError> {
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
}

struct PersonRoute;
#[async_trait]
impl Route for PersonRoute {
    fn endpoint(&self) -> &str {
        "/people/:name"
    }

    async fn handle(&self, params: &Params, _request: Request) -> Result<Response, AnyError> {
        let name = params.find("name").into_gem()?;

        Ok(Response::success(
            "text/gemini",
            format!(
                "# {name} - A custom page for you

This page was curated specifically for you \"{name}\", Congrats!
",
            ),
        ))
    }
}

struct PersonRouteSelect;
#[async_trait]
impl Route for PersonRouteSelect {
    fn endpoint(&self) -> &str {
        "/people"
    }

    async fn handle(&self, _params: &Params, request: Request) -> Result<Response, AnyError> {
        if !request.query.is_empty() {
            Ok(Response::redirect(format!(
                "{}{}/{}",
                request.script, request.path, request.query
            )))
        } else {
            Ok(Response::input("Enter your name"))
        }
    }
}

struct InfoRoute;
#[async_trait]
impl Route for InfoRoute {
    fn endpoint(&self) -> &str {
        "/info/*"
    }

    async fn handle(&self, _params: &Params, request: Request) -> Result<Response, AnyError> {
        let url = request.url;
        let addr = request.remote_addr;
        let host = request.remote_host;
        let path = request.path;
        let query = request.query;

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
}

#[tokio::main]
async fn main() {
    let mut app = RoutedApp::new();

    app.register(&MainRoute);
    app.register(&PersonRouteSelect);
    app.register(&PersonRoute);
    app.register(&InfoRoute);

    app.run_scgi("127.0.0.1:8000").await.unwrap();
}
