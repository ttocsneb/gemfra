use async_trait::async_trait;
use gemfra::{
    application::Application, error::AnyError, protocol::Cgi, request::Request, response::Response,
};

struct MyApp;

#[async_trait]
impl Application for MyApp {
    async fn handle_request(&self, request: Request) -> Result<Response, AnyError> {
        let url = request.url;
        let server_name = request.server_name;
        let ip = request.remote_addr;

        let query_message = match request.query {
            Some(query) => format!(
                "

You have given a query: {query}"
            ),
            None => format!(""),
        };

        let path_message = match !request.path.is_empty() {
            true => format!(
                "
You have requested for the path: {}",
                request.path
            ),
            false => format!(""),
        };

        Ok(Response::success(
            "text/gemini",
            format!(
                "# {server_name} - Request Info

Here, you will be able to get some information about your request for {url}!

I know your IP! it's {ip}, muahahaha!{path_message}{query_message}

There's not much else that I can share

"
            ),
        ))
    }
}

#[tokio::main]
async fn main() {
    MyApp.run_cgi().await;
}
