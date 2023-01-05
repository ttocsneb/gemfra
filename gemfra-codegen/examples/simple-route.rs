use gemfra::{error::AnyError, request::Request, response::Response};
use gemfra_codegen::route;

#[route("/")]
async fn my_route(_request: Request) -> Result<Response, AnyError> {
    Ok(Response::success("text/gemini", "# Hello World!"))
}

fn main() {}
