#![crate_id = "irc#0.1"]
#![crate_type = "lib"]

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::io::BufferedReader;
use std::io::IoError;

use msg::Message;
use msg::cmd;

pub mod msg;

pub struct IrcConnection<'a> {
	stream: TcpStream,
	output_sender: Sender<Message>,
	msg_callback: |&Message, &Sender<Message>|: 'a -> ()
}

impl<'a> IrcConnection<'a> {
	pub fn connect<'b>(
			host: &str, port: u16, nick: String, username: String,
			real_name: String, msg_callback: |&Message, &Sender<Message>|: 'b -> ()) -> IoResult<IrcConnection<'b>> {
		
		let (send_writer, rec_writer) = channel();

		let mut connection = IrcConnection {
			stream: try!(TcpStream::connect(host, port)),
			output_sender: send_writer.clone(),
			msg_callback: msg_callback,
		};

		let writer = connection.stream.clone();
		
		// spawn writer thread
		spawn(proc() {
			let mut writer = writer;
			for msg in rec_writer.iter() {
				(write!(writer, "{}", msg)).ok().expect("Unable to write to stream");
			}
		});

		connection.send(Message::new(cmd::Nick(nick)));
		connection.send(Message::new(cmd::User(username, 0, real_name)));
		Ok(connection)
	}

	pub fn send(&mut self, message: Message) {
		self.output_sender.send(message);
	}

	fn on_msg_rec(msg: &Message, sender: &Sender<Message>) {
		let prefix = &msg.prefix;
		let cmd = &msg.command;
		match *cmd {
			cmd::Ping(ref s) => sender.send(Message::new(cmd::Pong(s.clone()))),
			_ => { }
		};
	}

	pub fn run_loop(&mut self) {
		let reader = &mut self.stream;
		loop {
			fn reader_by_ref<'a, R: Reader>(reader: &'a mut R) -> std::io::RefReader<'a, R> { reader.by_ref() }
			
			reader.set_read_timeout(Some(500));
			let mut buf_reader = BufferedReader::new(reader_by_ref(reader));

			let line = buf_reader.read_line();
			match line {
				Ok(line) => match from_str::<Message>(line.as_slice().trim_right()) {
					Some(msg) => {
						IrcConnection::on_msg_rec(&msg, &self.output_sender);
						(self.msg_callback)(&msg, &self.output_sender);
					},
					None => println!("Invalid Message recieved"),
				},
				Err(IoError{kind: std::io::TimedOut, ..}) => continue,
				Err(e) => {
					println!("Unable to read line: {}", e);
					break;
				}
			}
		}
	}
}
