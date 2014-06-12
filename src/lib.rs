#![crate_id = "irc#0.1"]
#![crate_type = "lib"]

use std::io::TcpStream;
use std::io::BufferedReader;
use std::io::IoError;

use msg::Message;
use msg::cmd;

pub mod msg;

pub mod state {
	use msg::Message;
	use std::io::TcpStream;

	pub struct Disconnected;
	pub struct Connected {
		pub output: Sender<Message>,
		pub stream: TcpStream,
	}

	impl Drop for Connected {
		fn drop(&mut self) {
			let _ = self.stream.close_read();
			let _ = self.stream.close_write();
		}
	}
}

pub struct IrcClient<State> {
	nick: Option<String>,
	username: Option<String>,
	real_name: Option<String>,
	state: State,
}

impl IrcClient <state::Disconnected> {
	pub fn new(nick: String, username: String, real_name: String) -> IrcClient<state::Disconnected> {
		IrcClient { nick: Some(nick), username: Some(username), real_name: Some(real_name), state: state::Disconnected }
	}

	#[allow(experimental)]
	pub fn connect(mut self, host: &str, port: u16, message_sender: Sender<Message>) -> Result<IrcClient<state::Connected>, (IoError, IrcClient<state::Disconnected>)> {
		let stream = match TcpStream::connect(host, port) {
			Ok(stream) => stream,
			Err(e) => return Err((e, self))
		};

		let (send_writer, rec_writer) = channel();

		let connection = IrcClient{
			nick: self.nick.take(),
			username: self.username.take(),
			real_name: self.real_name.take(),
			state: state::Connected {
						 stream: stream.clone(),
						 output: send_writer.clone(),
					 }
		};

		let reader = stream.clone();
		let writer = stream;

		// spawn writer thread
		spawn(proc() {
			let mut writer = writer;
			for msg in rec_writer.iter() {
				(write!(writer, "{}\r\n", msg)).ok().expect("Unable to write to stream");
			}
		});

		spawn(proc() {
			let mut reader = reader;
			loop {
				fn reader_by_ref<'a, R: Reader>(reader: &'a mut R) -> std::io::RefReader<'a, R> { reader.by_ref() }
				
				reader.set_read_timeout(Some(500));
				let mut buf_reader = BufferedReader::new(reader_by_ref(&mut reader));

				let line = buf_reader.read_line();
				match line {
					Ok(line) => match from_str::<Message>(line.as_slice().trim_right()) {
						Some(msg) => {
							if message_sender.send_opt(msg.clone()).is_err() {
								break;
							}
							if on_msg_rec(&msg, &send_writer).is_err() {
								break;
							}
						},
						None => println!("Invalid Message recieved"),
					},
					Err(IoError{kind: std::io::TimedOut, ..}) => continue,
					Err(e) => {
						fail!("Unable to read line: {}", e);
					}
				}
			}
		});

		Ok(connection)
	}

}

impl IrcClient<state::Connected> {
	pub fn disconnect(self) -> IrcClient<state::Disconnected> {
		let IrcClient { nick: nick, username:username, real_name:real_name, .. } = self;
		IrcClient {
			state: state::Disconnected,
			nick: nick,
			username: username,
			real_name: real_name,
		}
	}

	pub fn send(&mut self, msg: Message) {
		self.state.output.send(msg);
	}

	pub fn sender<'a>(&'a self) -> &'a Sender<Message> {
		&self.state.output
	}
}

fn on_msg_rec(msg: &Message, sender: &Sender<Message>) -> Result<(), Message> {
	let _prefix = &msg.prefix;
	let cmd = &msg.command;
	match *cmd {
		cmd::Ping(ref s) => sender.send_opt(Message::new(cmd::Pong(s.clone()))),
		_ => Ok(())
	}
}
