extern crate irc;
extern crate getopts;
extern crate libc;

use std::os;
use std::io;

use getopts::{getopts, opt, optflag, optflagmulti, optmulti, optopt, reqopt, usage};

fn err(code: i32, err_msg: &str) -> ! {
	let mut stderr = io::stderr();
	writeln!(stderr, "{}", err_msg);
	unsafe{ libc::exit(code) }
}

fn main() {
	let opts = [
		optmulti("c", "channel", "What channel should be joined", "CHANNEL"),
		optflag("", "ignore-privmsg", "Ignore messages sent directly, only respond to those in a joinned channel"),
		optopt("p", "port", "What port should be used to connect to server", "PORT"),
	];
	let mut stderr = io::stderr();
	let mut args = os::args();
	let program = args.get(0);
	let matches = match getopts(args.tail(), opts) {
		Ok(m) => { m }
		Err(e) => { err(1, e.to_err_msg().as_slice()); }
	};

	let channels = matches.opt_strs("channel");
	let port: u16 = from_str(matches.opt_str("port").unwrap_or("6667".to_string()).as_slice()).expect("Please pass a port number to -p or --port");
	let ignore_privmsg = matches.opt_present("ignore-privmsg");
	let server = if matches.free.len() != 1 { err(2, "No server passed") } else { matches.free.move_iter().next().unwrap() };
	println!("server:{}, channels:{}, port:{}, ignore?:{}", server, channels, port, ignore_privmsg);
}
