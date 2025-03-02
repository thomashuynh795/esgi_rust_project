use std::collections::HashMap;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use shared::utils::{connect_to_server, register_player, register_team};

use crate::player::Player;

pub struct Team {
    pub name: String,
    pub registration_token: String,
    pub players: Vec<Player>,
    pub secrets: Arc<Mutex<HashMap<String, u64>>>,
    pub global_challenge: Arc<Mutex<Option<shared::types::challenge::Challenge>>>,
}

impl Team {
    pub fn register(
        server_address: &str,
        team_name: &String,
        members_number: usize,
    ) -> io::Result<Self> {
        let mut stream: TcpStream = connect_to_server(server_address)?;
        let registration_token: String = register_team(&mut stream, &team_name)?;
        let mut secrets: HashMap<String, u64> = HashMap::new();
        let mut global_challenge: Option<shared::types::challenge::Challenge> = None;
        for i in 0..members_number {
            let player_name: String = format!("Player {}", i + 1);
            secrets.insert(player_name, 0);
        }
        Ok(Team {
            name: String::from(team_name),
            registration_token,
            players: Vec::new(),
            secrets: Arc::new(Mutex::new(secrets)),
            global_challenge: Arc::new(Mutex::new(global_challenge)),
        })
    }

    pub fn add_player(&mut self, player_name: &str, server_address: &str) -> io::Result<()> {
        let mut stream: TcpStream = connect_to_server(server_address)?;

        let encoded_radar: String =
            register_player(&mut stream, &self.registration_token, player_name)?;

        let player: Player = Player::new(
            String::from(player_name),
            stream,
            encoded_radar,
            self.secrets.clone(),
            self.global_challenge.clone(),
        );

        self.players.push(player);

        return Ok(());
    }
}