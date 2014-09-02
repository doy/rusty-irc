#![crate_type = "lib"]

use std::io::TcpStream;
use std::io::BufferedReader;
use std::io::{IoError, IoResult};
use std::sync::{Arc, RWLock, Mutex};

use msg::Message;
use msg::cmd;

pub mod msg;

#[deriving(Clone, PartialEq, Eq)]
pub struct ClientConfig<T> {
	pub nicks: T,
	pub username: String,
	pub real_name: String,
}

struct SharedState<It> {
    username: String,
    real_name: String,
    future_nicks: Mutex<It>,
	chosen_nick: RWLock<String>,
}

pub struct IrcClient<T> {
	state: Arc<SharedState<T>>,
	stream: TcpStream,
	output_chan: Sender<Message>,
}

impl<T: Iterator<String>+Send+Sync> Clone for IrcClient<T> {
	fn clone(&self) -> IrcClient<T> {
		IrcClient {
			state: self.state.clone(),
			stream: self.stream.clone(),
			output_chan: self.output_chan.clone(),
		}
	}
}

impl<T: Iterator<String>+Send+Sync> IrcClient<T> {
	pub fn new(config: ClientConfig<T>, host: &str, port: u16, msg_chan: Sender<Message>) -> IoResult<IrcClient<T>> {
        let ClientConfig { mut nicks, username, real_name } = config;
		let stream = try!(TcpStream::connect(host, port));

		let (send_writer, rec_writer) = channel();

		let chosen_nick = nicks.next().unwrap();

		let state = SharedState {
            username: username.clone(),
            real_name: real_name.clone(),
            future_nicks: Mutex::new(nicks),
			chosen_nick: RWLock::new(chosen_nick.clone())
		};

		let connection = IrcClient {
			state: Arc::new(state),
			stream: stream.clone(),
			output_chan: send_writer.clone()
		};
		
		let reader = stream.clone();
		let writer = stream;

		// spawn writer thread
		std::task::TaskBuilder::new().named("rusty-irc:writer").spawn(proc() {
			let mut writer = writer;
			for msg in rec_writer.iter() {
				(write!(writer, "{}\r\n", msg)).ok().expect("Unable to write to stream");
			}
		});

        let reader_client = connection.clone();
		std::task::TaskBuilder::new().named("rusty-irc:reader").spawn(proc() {
			let mut reader = BufferedReader::new(reader);
            let mut reader_client = reader_client;
			loop {
				unsafe {
					let raw: *mut TcpStream = reader.get_ref() as *const _ as *mut _;
					(*raw).set_read_timeout(Some(500));
				}

				let line = reader.read_line();
				match line {
					Ok(line) => match from_str::<Message>(line.as_slice().trim_right()) {
						Some(msg) => {
							if msg_chan.send_opt(msg.clone()).is_err() {
								break;
							}
                            reader_client.on_msg_rec(&msg);
						},
						None => println!("Invalid Message recieved"),
					},
					Err(IoError{kind: std::io::TimedOut, ..}) => continue,
                    Err(IoError{kind: std::io::EndOfFile, ..}) => fail!("Connection closed by server"),
					Err(e) => fail!("Unable to read line: {}", e)
				}
			}
		});

        connection.send(Message::new(cmd::Nick(chosen_nick)));
        connection.send(Message::new(cmd::User(username, 8, real_name)));

		Ok(connection)
	}

	#[inline]
	pub fn send(&self, msg: Message) {
		let _ = self.output_chan.send_opt(msg);
	}

	pub fn sender<'a>(&'a self) -> &'a Sender<Message> {
		&self.output_chan
	}

	pub fn nick(&self) -> String {
		self.state.chosen_nick.read().clone()
	}

    pub fn username<'a>(&'a self) -> &'a str {
        self.state.username.as_slice()
    }

    pub fn real_name<'a>(&'a self) -> &'a str {
        self.state.real_name.as_slice()
    }

    fn on_msg_rec(&mut self, msg: &Message) {
        let _prefix = &msg.prefix;
        let cmd = &msg.command;
        match *cmd {
            cmd::Ping(ref s) => 
                self.send(Message::new(cmd::Pong(s.clone()))),
            cmd::Numeric(443, _, _) => {
                *self.state.chosen_nick.write() = self.state.future_nicks.lock().next().unwrap();
            },
            _ => ()
        }
    }
}

