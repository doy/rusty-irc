extern crate irc;
extern crate getopts;

use std::os;
use std::io;

use getopts::{getopts, opt, optflag, optflagmulti, optmulti, optopt, reqopt, usage};

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
		Err(e) => { let _ = writeln!(stderr, "{}", e.to_err_msg()); os::set_exit_status(1); return; }
	};

	let channels = matches.opt_strs("channel");
	let port: u16 = from_str(matches.opt_str("port").unwrap_or("6667".to_string()).as_slice()).expect("Please pass a port number to -p or --port");
	let ignore_privmsg = matches.opt_present("ignore-privmsg");
	println!("{}, {}, {}", channels, port, ignore_privmsg);
}
