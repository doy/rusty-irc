extern crate irc;

use std::io::stdio;

use irc::IrcClient;
use irc::msg::Message;
//use irc::msg::cmd;

fn main() {
	let mut stderr = stdio::stderr();

	let mut args = std::os::args().move_iter();
	args.next();
	let host = args.next().expect("No hostname passed");
	let port: u16 = from_str(args.next().unwrap_or_else(|| { let _ = writeln!(stderr, "No port given. Assuming 6667."); "6667".to_string() }).as_slice())
		.expect("Port must be a number");

	drop(args);

	let mut connection = IrcClient::connect(host.as_slice(), port, "rusty-irc".to_string(), "dremann".to_string(), "Zachary Dremann".to_string()).unwrap();
	let sender = connection.sender();

	let on_msg = |message: &Message| {
		println!("{}", *message);
	};
	
	spawn(proc() {
		let mut stdin = stdio::stdin();
		for line in stdin.lines() {
			match line {
				Ok(s) => {
					match from_str(s.as_slice()) {
						Some(msg) => sender.send(msg),
						None => ()
					}
				}
				Err(_) => break,
			}
		}
	});

	connection.run_loop(on_msg);
}
