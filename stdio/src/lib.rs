//! jsonrpc server using stdin/stdout
//!
//! ```no_run
//!
//! use jsonrpc_stdio_server::ServerBuilder;
//! use jsonrpc_stdio_server::jsonrpc_core::*;
//!
//! fn main() {
//! 	let mut io = IoHandler::default();
//! 	io.add_method("say_hello", |_params| {
//! 		Ok(Value::String("hello".to_owned()))
//! 	});
//!
//! 	ServerBuilder::new(io).build();
//! }
//! ```

#![warn(missing_docs)]

use std::io::Write;
use tokio;
use tokio_stdin_stdout;
#[macro_use]
extern crate log;

pub use jsonrpc_core;

use jsonrpc_core::IoHandler;
use std::sync::Arc;
use tokio::prelude::{Future, Stream};
use tokio_codec::{FramedRead, LinesCodec};

/// Stdio server builder
pub struct ServerBuilder {
	handler: Arc<IoHandler>,
}

impl ServerBuilder {
	/// Returns a new server instance
	pub fn new<T>(handler: T) -> Self
	where
		T: Into<IoHandler>,
	{
		ServerBuilder {
			handler: Arc::new(handler.into()),
		}
	}

	/// Will block until EOF is read or until an error occurs.
	/// The server reads from STDIN line-by-line, one request is taken
	/// per line and each response is written to STDOUT on a new line.
	pub fn build(&self) {
		let stdin = tokio_stdin_stdout::stdin(0);

		let framed_stdin = FramedRead::new(stdin, LinesCodec::new());

		let handler = self.handler.clone();
		let future = framed_stdin
                    .map_err(|e| panic!("{:?}", e))
                    .for_each(move |line| {
			let f = process(&handler, line)
                            .map(|s| {
                                println!("{}", s);
                            });

			tokio::spawn(f)
                    });

                tokio::run(future);
	}
}

/// Process a request asynchronously
fn process(io: &Arc<IoHandler>, input: String) -> impl Future<Item = String, Error = ()> + Send {
	io.handle_request(&input).map(move |result| match result {
		Some(res) => res,
		None => {
			info!("JSON RPC request produced no response: {:?}", input);
			String::from("")
		}
	})
}
