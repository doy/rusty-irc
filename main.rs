extern crate irc;

use std::io::net::tcp::TcpStream;
use std::io::BufferedReader;

use irc::IrcConnection;
use irc::msg::Message;
use irc::msg::cmd;

fn main() {
	let message = Message { 
		prefix: None,
		command: cmd::PrivMsg("#rust".to_string(), "Hi there everyone".to_string()),
	};
	
	println!("{}", message);

	let on_msg = |message: &Message, _sender: &Sender<Message>| {
		println!("{}", *message);
	};

	let mut connection = IrcConnection::connect("irc.mozilla.org", 6667, "Dr-Emann".to_string(), "dremann".to_string(), "Zachary Dremann".to_string(), on_msg).unwrap();

	connection.run_loop();
}
