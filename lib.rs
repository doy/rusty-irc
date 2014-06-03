//#![crate_id = "irc#0.1"]
//#![crate_type = "lib"]

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::io::BufferedReader;
use std::fmt;
use std::from_str::FromStr;

pub struct IrcConnection {
	callbacks: IrcCallbacks,
	stream: TcpStream
}

#[deriving(Clone)]
pub enum Command {
	Nick,
	User,
	Quit,
	Join,
	Part,
	PrivMsg,
	Notice,
	Motd,
	Ping,
	Pong,
	Error,
	Away,
	Numeric(u16),
	UnknownStr(String)
}

impl fmt::Show for Command {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "{}",
			match *self {
				Nick => "NICK".into_maybe_owned(),
				User => "USER".into_maybe_owned(),
				Quit => "QUIT".into_maybe_owned(),
				Join => "JOIN".into_maybe_owned(),
				Part => "PART".into_maybe_owned(),
				PrivMsg => "PRIVMSG".into_maybe_owned(),
				Notice => "NOTICE".into_maybe_owned(),
				Motd => "MOTD".into_maybe_owned(),
				Ping => "PING".into_maybe_owned(),
				Pong => "PONG".into_maybe_owned(),
				Error => "ERROR".into_maybe_owned(),
				Away => "AWAY".into_maybe_owned(),
				Numeric(i) => i.to_str().into_maybe_owned(),
				UnknownStr(ref s) => s.as_slice().into_maybe_owned(),
			}
		)
	}
}

impl FromStr for Command {
	fn from_str(s: &str) -> Option<Command> {
		Some(match s {
			"NICK" => Nick,
			"USER" => User,
			"QUIT" => Quit,
			"JOIN" => Join,
			"PART" => Part,
			"PRIVMSG" => PrivMsg,
			"NOTICE" => Notice,
			"MOTD" => Motd,
			"PING" => Ping,
			"PONG" => Pong,
			"ERROR" => Error,
			"AWAY" => Away,
			other => match from_str::<u16>(other) {
				Some(i) => Numeric(i),
				None => UnknownStr(other.to_string())
			}
		})
	}
}

#[deriving(Clone)]
pub struct Message {
	pub prefix: Option<String>,
	pub command: Command,
	pub arguments: Vec<String>,
}

impl<'a> fmt::Show for Message {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		if self.prefix.is_some() {
			try!(write!(formatter, ":{} ", self.prefix.get_ref()));
		}
		try!(write!(formatter, "{}", self.command));
		for argument in self.arguments.iter() {
			try!(write!(formatter, " {}", *argument));
		}
		Ok(())
	}
}

impl FromStr for Message {
	fn from_str(s: &str) -> Option<Message> {
		//TODO: Parse string
		unimplemented!();
	}
}



impl IrcConnection {
	pub fn connect(
		callbacks: IrcCallbacks, host: &str, port: u16,
		nick: &str,  username: &str, real_name: &str) -> IoResult<IrcConnection> {
		let mut stream = try!(TcpStream::connect(host, port));

		Ok(IrcConnection {
			callbacks: callbacks,
			stream: stream
		})
	}

	pub fn start_loop(&mut self) {
		let stream = self.stream.clone();
//		spawn(proc() {
			let mut reader = BufferedReader::new(stream);
			loop {
				match reader.read_line() {
					Ok(line) => {
						let mut words: Vec<&str> = line.as_slice().words().collect();

						println!("{}", words);
						if words.is_empty() { continue; }
						if (*words.get(0)).starts_with(":") {
							words.shift();
						}
						match *words.get(0) {
							"PING" => println!("PING"),
							other => println!("other: {}", other),
						}
					}
					Err(e) => {
						println!("An Error occured: {}", e);
						break;
					}
				}
				break;
			}
//		});
	}
}

pub struct IrcCallbacks {
	pub on_connect: fn(&mut IrcConnection)->(),
	pub on_numeric: fn(&mut IrcConnection, uint, &str, &[&str])->(),
	//TODO: Add the rest
}
