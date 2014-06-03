use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;

use lib::IrcCallbacks;
use lib::IrcConnection;

mod lib;

fn main() {
	fn on_connect(_connection: &mut IrcConnection) {
		println!("Connected");
	}
	fn on_numeric(_connection: &mut IrcConnection, n: uint, origin: &str, params: &[&str]) {
		println!("Numeric event \\#{} with params {}", n, params);
	}
	let callbacks = IrcCallbacks {
		on_connect: on_connect,
		on_numeric: on_numeric,
	};
	let mut connection = IrcConnection::connect(callbacks, "irc.mozilla.org", 6667, "Dr-Emann", "dremann", "Zachary Dremann").unwrap();

	connection.start_loop();

}
