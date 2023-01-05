//! Macros for gemfra
//!
//! ## [route](macro@route) macro
//!
//! A macro that allows you to write routes for a [RoutedApp](gemfra::routed::RoutedApp).
//!
//! ```
//! use gemfra::{
//!     response::Response,
//!     request::Request,
//!     error::AnyError,
//! };
//! use gemfra_codegen::route;
//!
//! #[route("/foo/:bar")]
//! async fn my_route(request: Request, bar: &str) -> Result<Response, AnyError> {
//!     todo!()
//! }
//! ```

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, FnArg, Item, LitStr, Type};

/// Convert the provided route into a struct that implements [Route](gemfra::routed::Route).
///
/// The macro should get an endpoint that the route will handle. This can have
/// variables which can be passed to the route function.
///
/// The route function will need a parameter named `request` and can optionally
/// have parameters specified by the endpoint. Internally, the route function is
/// async, so it does not matter whether the route function is marked async.
///
/// The endpoint can contain four kinds of segments:
///
/// * __segments__: these are of the format `/a/b`.
/// * __params__: these are of the format `/a/:b`.
/// * __named wildcards__: these are of the format `/a/*b`.
/// * __unnamed wildcards__: these are of the format `/a/*`.
///
/// Only params and named wildcards can be passed to the route function. By default,
/// a parameter is of type `&str`. You can however specify any type that impls
/// [FromStr](std::str::FromStr). The param will be parsed, and if it fails, a
/// `51 File not found` will be sent.
///
/// > Note that currently, it is not possible to have mutliple routes with the
/// > same endpoint, but different parameter types.
///
/// ### Examples
///
/// ```
/// use gemfra::{
///     response::Response,
///     request::Request,
///     error::AnyError,
/// };
/// use gemfra_codegen::route;
///
/// #[route("/foo/bar")]
/// async fn no_params(_request: Request) -> Result<Response, AnyError> {
///     Ok(Response::success("text/gemini", "# Hello World!"))
/// }
///
/// #[route("/foo/:my_var")]
/// async fn default_param(_request: Request, my_var: &str) -> Result<Response, AnyError> {
///     Ok(Response::success("text/gemini", format!("# Hello {my_var}")))
/// }
///
/// #[route("/foo/:year")]
/// async fn typed_param(_request: Request, year: i32) -> Result<Response, AnyError> {
///     // Any non i32 value for year will result in a `51 File not found`
///     Ok(Response::success("text/gemini", format!("# The year is {year}")))
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let endpoint = parse_macro_input!(args as LitStr);

    let endpoint_val = endpoint.value();
    let mut param_names = HashSet::new();
    for segment in endpoint_val.split("/") {
        if segment.starts_with(":") || segment.starts_with("*") {
            if segment == "*" {
                // We don't want unnamed
                continue;
            }
            if !(param_names.insert(segment[1..].to_owned())) {
                abort!(
                    endpoint.span(),
                    "Cannot have multiple named parameters with the same name";
                    help = "Rename or remove one of the parameters named `{}`", &segment[1..]
                );
            }
        }
    }

    let input = parse_macro_input!(input as Item);

    // Extract the function from the input
    let func = match &input {
        Item::Fn(f) => f,
        _ => {
            abort!(input.span(), "You can only use route on functions");
        }
    };
    let name = &func.sig.ident;
    let return_ty = &func.sig.output;
    let block = &func.block;

    // Extract all the parameters
    let mut request_arg = None;
    let mut params = Vec::new();
    for arg in &func.sig.inputs {
        if let FnArg::Typed(arg) = arg {
            if let syn::Pat::Ident(ident) = arg.pat.as_ref() {
                let mut arg_name = ident.ident.to_string();
                if arg_name.starts_with("_") {
                    arg_name.remove(0);
                }
                if arg_name == "request" {
                    request_arg = Some(arg);
                } else {
                    if !param_names.contains(&arg_name) {
                        abort!(
                            arg.span(), "Parameter `{}` not in endpoint", arg_name;
                            note = endpoint.span() => "Add `{}` to the endpoint", arg_name
                        );
                    }

                    let ty = &arg.ty;
                    let param_lit = LitStr::new(&arg_name, ident.ident.span());

                    let get_param = quote! {
                        gemfra::error::ToGemError::into_gem(params.find(#param_lit))?
                    };

                    // If the type is `&str`, we don't need to parse the value
                    if let Type::Reference(r) = ty.as_ref() {
                        if let Type::Path(path) = r.elem.as_ref() {
                            if let Some(segment) = path.path.segments.first() {
                                if segment.ident.to_string() == "str" {
                                    params.push(quote_spanned! {arg.span()=>
                                        let #ident: #ty = #get_param;
                                    });
                                    continue;
                                }
                            }
                        }
                    }

                    // Parse the type into the requested type
                    params.push(quote_spanned! {arg.span()=>
                        let #ident: #ty = gemfra::error::ToGemError::into_gem_type(
                            #get_param.parse(),
                            gemfra::error::GemErrorType::NotFound
                        )?;
                    });
                }
            }
        }
    }
    let request_arg = match request_arg {
        Some(v) => v,
        None => {
            abort!(func.sig.span(), "input `request` is a required parameter");
        }
    };

    TokenStream::from(quote! {
        #[allow(non_camel_case_types)]
        struct #name;

        #[async_trait::async_trait]
        impl gemfra::routed::Route for #name {
            fn endpoint(&self) -> &str {
                #endpoint
            }

            async fn handle(&self, params: &gemfra::routed::Params, #request_arg) #return_ty {
                #(#params)*
                #block
            }
        }
    })
}
