extern crate irc;
extern crate libc;

use std::io::stdio;

use irc::IrcClient;

fn main() {
	let mut stderr = stdio::stderr();

	let mut args = std::os::args().move_iter();
	args.next();
	let host = args.next().expect("No hostname passed");
	let port: u16 = from_str(args.next().unwrap_or_else(|| { let _ = writeln!(stderr, "No port given. Assuming 6667."); "6667".to_string() }).as_slice())
		.expect("Port must be a number");

	drop(args);

	let (tx, rx) = channel();

	let nicks = (vec!["rusti-irc".to_string()]).move_iter();
	let config = irc::ClientConfig {
		nicks: nicks,
		username: "dremann".to_string(),
		real_name: "Zachary Dremann".to_string()
	};

	let client = IrcClient::new(config, host.as_slice(), port, tx).unwrap();
	let sender = client.sender().clone();

	spawn(proc() {
		let mut stdin = stdio::stdin();
		for line in stdin.lines() {
			match line {
				Ok(s) => {
					match from_str(s.as_slice()) {
						Some(msg) => { if sender.send_opt(msg).is_err() { break; } },
						None => ()
					}
				}
				Err(_) => break,
			}
		}
	});

	for msg in rx.iter() {
		let c = client.clone();
		println!("{}", c.nick());
		println!("{} {}", msg.prefix, msg.command);
	}

	unsafe { libc::exit(0); }
}
