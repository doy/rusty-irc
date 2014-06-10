#![crate_id = "irc#0.1"]
#![crate_type = "lib"]

use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use std::io::BufferedReader;
use std::io::IoError;

use msg::Message;
use msg::cmd;

pub mod msg;

pub struct IrcClient {
	stream: TcpStream,
	output_sender: Sender<Message>,
}

impl IrcClient {
	pub fn connect(
			host: &str, port: u16, nick: String, username: String, real_name: String) -> IoResult<IrcClient> {
		
		let (send_writer, rec_writer) = channel();

		let mut connection = IrcClient{
			stream: try!(TcpStream::connect(host, port)),
			output_sender: send_writer.clone(),
		};

		let writer = connection.stream.clone();
		
		// spawn writer thread
		spawn(proc() {
			let mut writer = writer;
			for msg in rec_writer.iter() {
				(write!(writer, "{}\r\n", msg)).ok().expect("Unable to write to stream");
			}
		});

		connection.send(Message::new(cmd::Nick(nick)));
		connection.send(Message::new(cmd::User(username, 8, real_name)));
		Ok(connection)
	}

	pub fn send(&mut self, message: Message) {
		self.output_sender.send(message);
	}

	pub fn sender(&self) -> Sender<Message> {
		self.output_sender.clone()
	}

	fn on_msg_rec(msg: &Message, sender: &Sender<Message>) {
		let _prefix = &msg.prefix;
		let cmd = &msg.command;
		match *cmd {
			cmd::Ping(ref s) => sender.send(Message::new(cmd::Pong(s.clone()))),
			_ => { }
		};
	}

	#[allow(experimental)]
	pub fn run_loop(&mut self, on_msg: |&Message| -> ()) {
		let reader = &mut self.stream;
		loop {
			fn reader_by_ref<'a, R: Reader>(reader: &'a mut R) -> std::io::RefReader<'a, R> { reader.by_ref() }
			
			reader.set_read_timeout(Some(500));
			let mut buf_reader = BufferedReader::new(reader_by_ref(reader));

			let line = buf_reader.read_line();
			match line {
				Ok(line) => match from_str::<Message>(line.as_slice().trim_right()) {
					Some(msg) => {
						IrcClient::on_msg_rec(&msg, &self.output_sender);
						on_msg(&msg);
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
