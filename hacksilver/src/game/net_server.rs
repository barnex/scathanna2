use super::internal::*;
use std::sync::mpsc::TryRecvError;

/// Network RPC & driver layer on top of `ServerState`.
///
/// Handles:
///		Listening for incoming connections
/// 	Dropping players from disconnected connections
/// 	Ticking `Severstate` main loop
/// 	Forwarding `ServerMsg`s to subsets of clients (Just(player)/Not(player)/All)
///
pub struct NetServer {
	listen: Receiver<TcpStream>, // incoming connections are sent here
	clients: HashMap<ID, NetPipe>,
	tick_duration: Duration,

	state: ServerState,
}

type NetPipe = crate::net::NetPipe<ServerMsg, ClientMsg>;

impl NetServer {
	/// Serve incoming connections on `opts.addr`.
	/// Only returns in case of error.
	pub fn listen_and_serve(opts: ServerOpts) -> Result<()> {
		Self::new(opts)?.serve_loop()
	}

	fn new(opts: ServerOpts) -> Result<Self> {
		let listen_for_conn = Self::spawn_listen_loop(&opts.addr)?;
		let tick_duration = Duration::from_millis(30); // TODO

		Ok(Self {
			listen: listen_for_conn,
			clients: HashMap::default(),
			state: ServerState::new(opts)?,
			tick_duration,
		})
	}

	fn serve_loop(&mut self) -> Result<()> {
		loop {
			let start = Instant::now();

			self.tick()?;

			let now = Instant::now();
			let elapsed = now - start;
			if let Some(sleep) = self.tick_duration.checked_sub(elapsed) {
				thread::sleep(sleep)
			}
		}
	}

	fn tick(&mut self) -> Result<()> {
		self.tick_listen()?;
		self.tick_client_msgs()?;
		let diffs = self.state.handle_tick(self.tick_duration.as_secs_f32());
		self.flush_diffs(diffs);
		Ok(())
	}

	//-------------------------------------------------------------------------------- accept new clients

	// check for incoming connections (non-blocking)
	fn tick_listen(&mut self) -> Result<()> {
		match self.listen.try_recv() {
			Ok(tcp_stream) => Ok(self.handle_conn(tcp_stream)),
			Err(TryRecvError::Empty) => Ok(()),
			Err(TryRecvError::Disconnected) => Err(anyhow!("server: listen thread died")),
		}
	}

	// Handle a new client connection.
	fn handle_conn(&mut self, tcp_stream: TcpStream) {
		if let Err(e) = self.handle_conn_with_result(tcp_stream) {
			error!("handle_conn: error: {}", e)
		}
	}

	// add new player to the game, send them the full state.
	fn handle_conn_with_result(&mut self, mut tcp_stream: TcpStream) -> Result<()> {
		// Perform a handshake:
		//  * Client sends JoinMsg with player info
		//  * Server sends AcceptMsg with client ID and map to load.
		let join_msg: JoinRequest = wireformat::deserialize_from(&mut tcp_stream)?;
		let name = join_msg.name.clone();
		let (player_id, map_switch) = self.state.join_new_player(join_msg);
		info!("accepting {:?} ({:?}) as {}", tcp_stream.local_addr().unwrap(), name, player_id);
		wireformat::serialize_into(&mut tcp_stream, &AcceptedMsg { player_id, map_switch })?;
		let pipe = NetPipe::new(tcp_stream);
		assert!(self.clients.insert(player_id, pipe).is_none());
		Ok(())
	}

	//-------------------------------------------------------------------------------- client messages

	// Incoming client messages, if any, are forwarded to `ServerState` for handling.
	// Closed connections or bad wire data cause the client to be dropped and removed from the game.
	fn tick_client_msgs(&mut self) -> Result<()> {
		let mut drop = vec![];
		for (id, pipe) in &mut self.clients {
			while let Some(msg) = pipe.try_recv() {
				match msg {
					Ok(msg) => self.state.handle_client_msg(*id, msg),
					Err(e) => {
						error!("error reading from client {id}: {e}, dropping client.");
						drop.push(*id);
						break;
					}
				}
			}
		}

		for &id in &drop {
			self.handle_drop_client(id)
		}

		Ok(())
	}

	//-------------------------------------------------------------------------------- clients disconnect

	// Handle a dropped connection event.
	fn handle_drop_client(&mut self, client_id: ID) {
		info!("dropping client {client_id} ({} left)", self.clients.len());
		self.clients.remove(&client_id);
		self.state.handle_drop_player(client_id);
		//let diffs = self.state.take_diffs();
		//self.flush_diffs(diffs); // needed?
	}

	//____________________________________________________________ communication protocol

	fn flush_diffs(&mut self, diffs: Diffs) {
		let client_ids = self.clients.keys().copied().collect::<SmallVec<[ID; 16]>>();
		for msg in diffs.into_iter() {
			for client_id in Self::addressees(&client_ids, msg.to) {
				self.send_to(client_id, msg.msg.clone())
			}
		}
	}

	// expand Addressee (Just/Not/All) into list of matching client IDs.
	fn addressees(clients: &[ID], a: Addressee) -> SmallVec<[ID; 8]> {
		match a {
			Addressee::Just(id) => smallvec![id],
			Addressee::Not(id) => clients.into_iter().copied().filter(|&i| i != id).collect(),
			Addressee::All => clients.iter().copied().collect(),
		}
	}

	// send a message to just one player
	fn send_to(&mut self, player_id: ID, msg: ServerMsg) {
		if let Some(client) = self.clients.get_mut(&player_id) {
			match client.send(msg) {
				Err(e) => {
					error!("send_to {player_id}: {e}");
					self.handle_drop_client(player_id)
				}
				Ok(()) => (),
			}
		}
	}

	//____________________________________________________________ async workers

	// Spawn a loop that accepts incoming TCP connections on `address`,
	// sends them over a channel for non-blocking access by the server's main thread.
	fn spawn_listen_loop(address: &str) -> Result<Receiver<TcpStream>> {
		let (send, recv) = channel();
		println!("------------------------------------");
		println!(" Listening on {address}");
		println!("------------------------------------");
		let listener = TcpListener::bind(address)?;
		thread::spawn(move || {
			for stream in listener.incoming() {
				match stream {
					Err(e) => error!("accept: {e}"), // client failed to connect, server carries on.
					Ok(tcp_stream) => {
						info!("accepted connection {}", tcp_stream.peer_addr().unwrap());
						if send.send(tcp_stream).is_err() {
							info!("listen: quitting");
							return; // server quit, so stop worker thread.
						}
					}
				}
			}
		});
		Ok(recv)
	}
}
