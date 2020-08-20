use tokio::prelude::*;
use tokio::net::{TcpStream, TcpListener};
use tokio::fs::File;
use std::error::Error;
use std::net::{SocketAddr, Shutdown};
use dotenv::{from_filename, var};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};

struct Client {
    stream:     TcpStream,
    addr:       SocketAddr,
    failed:     u8
}

impl Client {
    pub fn new(raw : (TcpStream, SocketAddr)) -> Client {
        Client {
            stream: raw.0,
            addr:   raw.1,
            failed: 0
        }
    }

    pub async fn login(&mut self, passwd: &String) -> bool {
        // Get password size
        let client_passwd_size_result = self.stream.read_u8().await;

        if client_passwd_size_result.is_err() {
            return false; // The Login Process has failed if there is no size
        }

        // Create a password buffer with the according size
        let mut client_passwd_buffer: Vec<u8> = vec![0 as u8; client_passwd_size_result.unwrap() as usize];

        // Read the bytes into the password buffer
        if self.stream.read_exact(&mut client_passwd_buffer).await.is_err() {
            return false; // The Login Process has failed if there is no password
        }

        // Create a String from the password buffer's bytes
        let client_passwd_result = String::from_utf8(client_passwd_buffer);

        if client_passwd_result.is_err() {
            return false; // The Login Process has failed if the password is not formatted correctly
        }

        // If the client provided password matches the server's password, the login was successful
        return client_passwd_result.unwrap() == passwd;
    }
}

#[tokio::main]
pub async fn main() {
    dotenv::from_filename("./config.env");

    let server_password_result = var("MULTIFACTORY_PASSWD");
    let mut server_password: String = String::new();

    // If there is no existing env file with a password, generate one
    if server_password_result.is_err() {
        // TODO: Generate Server Password if not given
    } else {
        server_password = server_password_result.unwrap();
    }

    let mut blocked_ips: Vec<SocketAddr> = Vec::new();

    let is_hosted: bool     = false;
    let hosted_by: String   = String::new();

    //
    // "Log In" Procedure:
    //  1. Client Sends Password to Server
    //  2. Server responds:
    //      '0' if the password is correct
    //           and nobody is hosting
    //      '1' if the password is correct
    //           but someone is already hosting the world
    //      '2' if the password is incorrect
    //           (in which case a counter gets incremented up to
    //           MAX_LOGIN_ATTEMPTS (DEFAULT=3), where the ip will
    //           be blocked until removed from the blocked ip's)
    //  3. Server provides further information:
    //      - If someone is already hosting, it will send their name
    //         to the client
    //      - If nobody is hosting, it will send the latest World save
    //

    // Create a TCP Listener
    let mut listener = TcpListener::bind("localhost:8085").await.unwrap();

    // Listen for incoming connections
    loop {
        // Accept an incoming connection
        let mut client = Client::new(listener.accept().await.unwrap());

        // If the connection's ip is blocked, ignore the connection
        if blocked_ips.contains(&client.addr) {
            continue;
        }

        // Try logging in with the connection for 3 times before blocking it
        while client.failed < 3 {
            if client.login(&server_password).await {
                break;
            }
            client.stream.write_u8(2).await;
            client.failed += 1;
        }

        // If login has failed three times, block the connection and listen for
        //  the next one
        if client.failed == 3 {
            blocked_ips.push(client.addr);
            continue;
        }

        // If the login has succeeded and nobody is hosting, send the latest world
        //  save to the client


    }

    config::test();
    /*
    //let mut passwd_buffer:Vec<u8> = vec![0; server_password_length];
    //let mut buffer_vec = vec![0 as u8; server_password_length];
    let mut buffer = [0;4];

    client.0.read(&mut buffer[..]).await.expect("didnt work :/");

    println!("received: {:?}", buffer);*/

}