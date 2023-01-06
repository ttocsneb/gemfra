[![crates-io](https://img.shields.io/crates/v/gemfra.svg)][crates-io]
[![api-docs](https://docs.rs/gemfra/badge.svg)][Documentation]

[crates-io]: https://crates.io/crates/gemfra
[Documentation]: https://docs.rs/gemfra

# A framework for writing gemini CGI scripts

Simplify the process of making CGI scripts for gemini with gemfra. This crate
abstracts most of the boilerplate involved in creating a CGI application. With
gemfra, you can choose which type of CGI protocol you want to run. 

Gemfra supports two CGI protocols:

* CGI - execute a script for every request  
* SCGI - run a server that handles CGI requests

These protocols will be implemented for applications. You can use an application
made from gemfra, or create your own application. Currently there is only one
pre-built application available:

* RoutedApp - Have multiple handlers that each handle a different route. 

## Async Runtime

Gemfra is an asynchronous library. It uses tokio as its runtime.

## Limitations

This software has not been tested with very many servers. It is possible, if not
likely that there are incompatibilities with other servers. I would like Gemfra 
to be as portable as possible, so please report any issues that you find.

## Roadmap

Gemfra is still in development and there are a few things that are planned to be
added to the library.

* FastCGI support (If there are any servers that support it)
* utilities for request/certificate
