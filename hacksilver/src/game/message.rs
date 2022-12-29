///! Client-Server messaging protocol.
use super::internal::*;

/// Message addressed to a (sub)set of all players.
pub struct Envelope<T> {
	pub to: Addressee,
	pub msg: T,
}

/// Who to send a message to: Just one player, All but one player, All players.
#[derive(Copy, Clone)]
pub enum Addressee {
	Just(ID),
	Not(ID),
	All,
}

pub type ClientMsgs = Vec<ClientMsg>;
pub type ServerMsgs = Vec<Envelope<ServerMsg>>;

/// Initial message sent by client when first joining a server.
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinRequest {
	pub name: String, // Player's nickname
	pub avatar_id: u8,
	pub team: Team,
}

/// Initial message sent by client when first joining a server.
#[derive(Serialize, Deserialize)]
pub struct AcceptedMsg {
	pub player_id: ID,
	pub map_switch: MapSwitch,
}

/// Subsequent messages sent by Client after the initial JoinMsg.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMsg {
	// The client controls their player movement as long as the player is spawned.
	// The server takes over when despawned.
	//
	// In a race where the server despawns but the client has not received the message,
	// the server will ignore the move request because not spawned server side..
	MovePlayerIfSpawned(Frame),

	// I'm ready to (re-)spawn
	ReadyToSpawn,

	// Spawn a visual effect.
	AddEffect(Effect),

	// Start a sound effect.
	PlaySound(SoundEffect),

	// I have shot player with ID `victim`.
	HitPlayer(ID),

	// Send a CLI command to the server.
	Command(String),
}

/// Messages sent by Server.
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMsg {
	/// Server tells client to add a new player to the game.
	AddPlayer(Player),

	/// Server tells client to remove a player from the game.
	DropPlayer(ID),

	/// Server tells client to change maps.
	/// (Server will first have de-spawned. Will force respawn after mapswitch).
	SwitchMap(MapSwitch),

	// ??? TODO: remove
	ForceMovePlayer(vec3),

	// Server tells client to update the position, orientation, velocity of *other* players.
	MovePlayer(ID, Frame),

	// Server tells client to update their player, except for position, orientation, velocity
	// which is controlled locally.
	UpdatePlayerPartial(Player),

	// Server tells client to update *everything*, even postion, orientation, velocity
	// which would normally be controlled locally. The server will only do so when the player
	// is despawned.
	UpdatePlayerFull(Player),

	AddEffect(Effect),
	PlaySound(SoundEffect),
	UpdateHUD(HUDUpdate),
	Log(String),
}

// Message requesting that the client switches to a new map.
// Conceptually, this messages encodes a `World` (map + entities),
// but we don't explicitly serialize the map (it's large) -- send the map name instead.
#[derive(Serialize, Deserialize, Clone)]
pub struct MapSwitch {
	pub map_name: String,
	pub entities: Entities,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HUDUpdate {
	pub pos: HUDPos,
	pub text: String,
	pub ttl_sec: f32,
}

impl ServerMsg {
	pub fn to_all(self) -> Envelope<Self> {
		self.to(Addressee::All)
	}

	pub fn to_just(self, client_id: ID) -> Envelope<Self> {
		self.to(Addressee::Just(client_id))
	}

	pub fn to_not(self, client_id: ID) -> Envelope<Self> {
		self.to(Addressee::Not(client_id))
	}

	pub fn to(self, to: Addressee) -> Envelope<Self> {
		Envelope { to, msg: self }
	}
}
