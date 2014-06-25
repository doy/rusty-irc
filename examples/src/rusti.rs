extern crate irc;
extern crate getopts;
extern crate libc;

use std::os;
use std::io;

use getopts::{getopts, opt, optflag, optflagmulti, optmulti, optopt, reqopt, short_usage, usage};

struct ShowableTraitObject<'a>(&'a std::fmt::Show);

impl<'a> std::fmt::Show for ShowableTraitObject<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let &ShowableTraitObject(inner) = self;
		inner.fmt(f)
	}
}

fn main() {

	let opts = [
		optflag("h", "help", "Print help information"),
		optmulti("c", "channel", "What channel should be joined", "CHANNEL"),
		optopt("p", "port", "What port should be used to connect to server", "PORT"),
		optflag("", "ignore-privmsg", "Ignore messages sent directly, only respond to those in a joinned channel"),
	];
	let args = os::args();
	let program = args.get(0);

	let err: |i32, &std::fmt::Show| -> ! = |code: i32, err_msg: &std::fmt::Show| {
		let mut stderr = io::stderr();
		let _ = writeln!(stderr, "{}", ShowableTraitObject(err_msg));
		let short = short_usage(program.as_slice(), opts);
		let _ = writeln!(stderr, "{}", usage(short.as_slice(), opts));
		unsafe{ libc::exit(code) }
	};

	let matches = match getopts(args.tail(), opts) {
		Ok(m) => { m }
		Err(e) => { err(1, &e); }
	};

	if matches.opt_present("help") {
		let short = short_usage(program.as_slice(), opts);
		println!("{}", usage(short.as_slice(), opts));
		return;
	}

	let channels = matches.opt_strs("channel");
	let port: u16 = from_str(matches.opt_str("port").unwrap_or("6667".to_string()).as_slice()).expect("Please pass a port number to -p or --port");
	let ignore_privmsg = matches.opt_present("ignore-privmsg");
	let server = if matches.free.len() != 1 { err(2, &"No server passed") } else { matches.free.move_iter().next().unwrap() };
	println!("server:{}, channels:{}, port:{}, ignore?:{}", server, channels, port, ignore_privmsg);
}
