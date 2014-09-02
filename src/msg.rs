use std::fmt;
use std::from_str::FromStr;
use std::ascii::OwnedStrAsciiExt;

pub mod cmd {
	#[deriving(Clone, PartialEq, Eq, Show, Hash)]
	pub enum Command {
		Nick(String),
		User(String, u8, String),
		Quit(Option<String>),
		Join(String),
		Part(String, Option<String>),
		PrivMsg(String, String),
		Notice(String, String),
		Motd(Option<String>),
		Ping(String),
		Pong(String),
		Error(String),
		Away(Option<String>),
		Numeric(u16, String, Vec<String>),
		UnknownCmd(String, Vec<String>)
	}
}

#[deriving(Clone, PartialEq, Eq, Hash)]
pub struct Prefix {
	pub name: String,
	pub user: Option<String>,
	pub host: Option<String>,
}

#[deriving(Clone, PartialEq, Eq, Hash)]
pub struct Message {
	pub prefix: Option<Prefix>,
	pub command: cmd::Command,
}

impl Message {
	pub fn new(command: cmd::Command) -> Message {
		Message { prefix: None, command: command }
	}

	pub fn with_prefix(prefix: Prefix, command: cmd::Command) -> Message {
		Message { prefix: Some(prefix), command: command }
	}
}

impl fmt::Show for Message {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		if self.prefix.is_some() {
			try!(write!(formatter, ":{} ", self.prefix.get_ref()));
		}

		match self.command {
			cmd::Nick(ref nickname) => write!(formatter, "NICK {}", nickname),
			cmd::User(ref username, mode, ref real_name) => write!(formatter, "USER {} {} * :{}", username, mode, real_name),
			cmd::Quit(ref msg) => if msg.is_some() { write!(formatter, "QUIT :{}", *msg.get_ref()) } else { write!(formatter, "QUIT") },
			cmd::Join(ref channel) => write!(formatter, "JOIN :{}", channel),
			cmd::Part(ref channel, ref msg) => if msg.is_some() { write!(formatter, "PART {} :{}", channel, *msg.get_ref()) } else { write!(formatter, "PART {}", channel) },
			cmd::PrivMsg(ref target, ref msg) => write!(formatter, "PRIVMSG {} :{}", target, msg),
			cmd::Notice(ref target, ref msg) => write!(formatter, "NOTICE {} :{}", target, msg),
			cmd::Ping(ref msg) => write!(formatter, "PING :{}", msg),
			cmd::Pong(ref msg) => write!(formatter, "PONG :{}", msg),
			cmd::Error(ref msg) => write!(formatter, "ERROR :{}", msg),
			cmd::Away(ref msg) => if msg.is_some() { write!(formatter, "AWAY :{}", msg.get_ref()) } else { write!(formatter, "AWAY") },
			cmd::Motd(ref target) => if target.is_some() { write!(formatter, "MOTD :{}", target.get_ref()) } else { write!(formatter, "MOTD") },
			cmd::Numeric(i, ref target, ref args) => {
				try!(write!(formatter, "{:03u} {}", i, target));
				let mut iter = args.iter().peekable();
				loop {
					match iter.next() {
						Some(arg) => {
							try!(
								if iter.peek().is_some() {
									write!(formatter, " {}", arg)
								}
								else {
									write!(formatter, " :{}", arg)
								}
							);
						}
						None => break
					}
				}
				Ok(())
			}
			cmd::UnknownCmd(ref cmd, ref args) => {
				try!(write!(formatter, "{}", cmd));
				let mut iter = args.iter().peekable();
				loop {
					match iter.next() {
						Some(arg) => {
							try!(
								if iter.peek().is_some() {
									write!(formatter, " {}", arg)
								}
								else {
									write!(formatter, " :{}", arg)
								}
							)
						}
						None => break
					}
				}
				Ok(())
			}
		}
	}
}

impl fmt::Show for Prefix {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(write!(f, "{}", self.name));
		if self.user.is_some() {
			try!(write!(f, "!{}", self.user.get_ref()));
		}
		if self.host.is_some() {
			try!(write!(f, "@{}", self.host.get_ref()));
		}
		Ok(())
	}
}

impl FromStr for Prefix {
	fn from_str(s: &str) -> Option<Prefix> {
		let name;
		let user;
		let host;
		match s.find('!') {
			Some(user_start) => {
				name = s.slice_to(user_start).to_string();
				let rest = s.slice_from(user_start + 1);
				match rest.find('@') {
					Some(host_start) => {
						user = Some(rest.slice_to(host_start).to_string());
						host = Some(rest.slice_from(host_start + 1).to_string());
					}
					None => {
						user = Some(rest.to_string());
						host = None;
					}
				}
			}
			None => {
				name = s.to_string();
				user = None;
				host = None;
			}
		}
		Some(Prefix{ name: name, user: user, host: host })
	}

}

impl FromStr for Message {
	fn from_str(s: &str) -> Option<Message> {
		let mut prefix = None;
		let mut cmd = None;
		let mut args = Vec::new();
		let mut current_str: Option<String> = None;
		let mut is_prefix = false;
		let mut is_final = false;
		
		for c in s.chars() {
			match c {
				c if is_final => {
					current_str.as_mut().unwrap().push_char(c);
				}
				' ' => {
					if is_prefix {
						prefix = current_str.take();
					}
					else if cmd.is_none() {
						cmd = current_str.take();
					}
					else {
						args.push(current_str.take().unwrap());
					}
					is_prefix = false;
				}
				':' if current_str.is_none() => {
					current_str = Some(String::new());
					if cmd.is_none() {
						is_prefix = true;
					}
					else {
						is_final = true;
					}
				}
				c => {
					if current_str.is_none() {
						current_str = Some(String::new());
					}
					current_str.as_mut().unwrap().push_char(c)
				}
			}
		}

		if cmd.is_none() {
			cmd = current_str.take();
		}
		else {
			args.push(current_str.take().unwrap());
		}

		let prefix: Option<Prefix> = prefix.and_then(|s| from_str(s.as_slice()));

		let cmd = match cmd.map(|s| s.into_ascii_upper()).as_ref().map(|s| s.as_slice()) {
			Some(s) => {
				match s {
					"NICK" if args.len() == 1 => cmd::Nick(args.pop().unwrap()),
					"USER" if args.len() == 4 => {
						let mut iter = args.move_iter();
						let uname = iter.next().unwrap();
						let opt_mode: Option<u8> = from_str(iter.next().unwrap().as_slice());
						iter.next();
						let fullname = iter.next().unwrap();
						cmd::User(uname, opt_mode.unwrap_or(0), fullname)
					}
					"NOTICE" if args.len() == 2 => {
						let mut iter = args.move_iter();
						cmd::Notice(iter.next().unwrap(), iter.next().unwrap())
					}
					"PRIVMSG" if args.len() == 2 => {
						let mut iter = args.move_iter();
						cmd::PrivMsg(iter.next().unwrap(), iter.next().unwrap())
					}
					"PING" if args.len() == 1 => cmd::Ping(args.pop().unwrap()),
					"PONG" if args.len() == 1 => cmd::Pong(args.pop().unwrap()),
					"AWAY" if args.len() == 0 || args.len() == 1 => cmd::Away(args.pop()),
					"QUIT" if args.len() == 0 || args.len() == 1 => cmd::Quit(args.pop()),
					"JOIN" if args.len() == 1 => cmd::Join(args.pop().unwrap()),
					"MOTD" if args.len() == 0 || args.len() == 1 => cmd::Motd(args.pop()),
					other => {
						match from_str::<u16>(other) {
							Some(n) if args.len() > 0 => {
								let target = args.remove(0).unwrap();
								cmd::Numeric(n, target, args)
							}
							_ => cmd::UnknownCmd(s.to_string(), args)
						}
					}
				}
			}
			None => cmd::UnknownCmd(String::new(), args)
		};

		Some(Message { prefix: prefix, command: cmd })
	}
}
