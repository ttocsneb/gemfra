use std::{env, error::Error};

use async_trait::async_trait;
use gemfra::{
    protocol::Cgi,
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

    async fn handle(&self, _params: &Params, request: Request) -> Result<Response, Box<dyn Error>> {
        let script = &request.script;
        Ok(Response::success(
            "text/gemini",
            format!(
                "# Hello World

Here is my example routed capsule!

=> {script}/vars View a variable
=> {script}/env View my environment variables


If you are running this command from the terminal, set the environment variable `PATH_INFO` to change the current path.

```
PATH_INFO=\"/env\" cargo run --example cgi-routed-demo
```
"
            ),
        ))
    }
}

struct VarRoute;
#[async_trait]
impl Route for VarRoute {
    fn endpoint(&self) -> &str {
        "/vars/:var"
    }

    async fn handle(&self, params: &Params, _request: Request) -> Result<Response, Box<dyn Error>> {
        let var = params.find("var").unwrap();

        Ok(Response::success(
            "text/gemini",
            format!(
                "# {var} - A custom page for you

This page was curated specifically for you \"{var}\", Congrats!
",
            ),
        ))
    }
}

struct VarRouteSelect;
#[async_trait]
impl Route for VarRouteSelect {
    fn endpoint(&self) -> &str {
        "/vars"
    }

    async fn handle(&self, _params: &Params, request: Request) -> Result<Response, Box<dyn Error>> {
        if !request.query.is_empty() {
            Ok(Response::redirect(format!(
                "{}{}/{}",
                request.script, request.path, request.query
            )))
        } else {
            Ok(Response::input("Enter a variable"))
        }
    }
}

struct EnvRoute;
#[async_trait]
impl Route for EnvRoute {
    fn endpoint(&self) -> &str {
        "/env"
    }

    async fn handle(
        &self,
        _params: &Params,
        _request: Request,
    ) -> Result<Response, Box<dyn Error>> {
        let mut vars = Vec::new();

        for (k, v) in env::vars() {
            vars.push(format!("{k}={v}"));
        }

        let vars = vars.join("\n");

        Ok(Response::success(
            "text/gemini",
            format!(
                "# Environment Variables

View my environment variables:

```
{vars}
```
",
            ),
        ))
    }
}

#[tokio::main]
async fn main() {
    /////////////////////////////////////////////////
    // set the needed environment variables if they are missing for cmd line use
    fn set_default(key: &str, default: &str) {
        if let Err(_) = env::var(key) {
            env::set_var(key, default);
        }
    }
    set_default("PATH_INFO", "");
    set_default("SCRIPT_NAME", "");
    set_default("SERVER_NAME", "localhost");
    set_default("QUERY_STRING", "");
    set_default("SERVER_PORT", "1965");
    set_default("GEMINI_URL", "gemini://localhost");
    set_default("REMOTE_ADDR", "127.0.0.1");
    set_default("REMOTE_HOST", "localhost");
    set_default("SERVER_PROTOCOL", "GEMINI");
    /////////////////////////////////////////////////

    let mut app = RoutedApp::new();

    app.register(&MainRoute);
    app.register(&VarRouteSelect);
    app.register(&VarRoute);
    app.register(&EnvRoute);

    app.run_cgi().await
}
