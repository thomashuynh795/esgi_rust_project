#![allow(unused)]
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8000")?;

    let mut s = "Hello😎😎";

    println!("{}", s.len());
    println!("{}", s.chars().count());

    println!("{:?}", s.as_bytes());
    let n = s.len() as u32;
    // 256
    // 00 00 01 00 : BigEndian
    // 00 01 00 00 : LittleEndian

    let bytes = n.to_le_bytes();
    // let bytes = u32::to_le_bytes(n); // version avec un type plus explicite
    println!("{bytes:?}");
    stream.write(&bytes)?;
    stream.write(s.as_bytes())?;
    Ok(())
} // the stream is closed here
