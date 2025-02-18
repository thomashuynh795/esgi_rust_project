use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    println!("Connection from {}", stream.peer_addr().unwrap());

    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer).unwrap();
    let n = u32::from_le_bytes(size_buffer);
    let mut buffer = vec![0; n as usize];
    stream.read_exact(&mut buffer).unwrap();
    println!("Buffer: {:x?}", buffer);

    let s = String::from_utf8(buffer.to_vec()).unwrap();
    println!("Request: {}", s);

    // let mut s = String::new();
    // stream.read_to_string(&mut s).unwrap();
    // println!("{}", s);
}

fn main() {
    shared::hello_world();

    match inner_main() {
        Ok(()) => {
            println!("Success")
        }
        Err(err) => {
            println!("Error: {err}");
        }
    }
}

fn inner_main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[test]
    fn test1() {
        #[derive(Serialize, Deserialize)]
        struct RegisterTeam {
            truc: u8,
        }
        let x = serde_json::to_string(&RegisterTeam { truc: 0 }).unwrap();
        println!("{}", x);
    }

    #[test]
    fn test2() {
        #[derive(Serialize, Deserialize)]
        enum Messages {
            RegisterTeam { team_name: String },
        }

        let x = serde_json::to_string(&Messages::RegisterTeam {
            team_name: "team".to_string(),
        })
        .unwrap();
        println!("{}", x);
    }
}
