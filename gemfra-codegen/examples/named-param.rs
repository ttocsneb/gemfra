use gemfra::{error::AnyError, request::Request, response::Response};
use gemfra_codegen::route;

#[route("/var/:year/:var")]
async fn my_route(_request: Request, year: i32, var: &str) -> Result<Response, AnyError> {
    Ok(Response::success(
        "text/gemini",
        format!(
            "# Hello World!
    
You've asked for the variable `{var}` in {year}
"
        ),
    ))
}

fn main() {}
