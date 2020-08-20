use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::fs::File;
use std::error::Error;
use dotenv::{from_filename, var};
use std::io::{stdin, stdout, Write};
use std::process::exit;

async fn login(passwd: &String, stream: &mut TcpStream) -> u8 {
    // Send password length
    stream.write_u8(passwd.len() as u8).await;

    // Send password
    stream.write(passwd.as_bytes()).await;

    // Listen for the Server's response
    let server_response = stream.read_u8().await;

    if server_response.is_err() {
        return 2;
    }

    return server_response.unwrap();
}

fn prepare_env() -> (String, String) {
    dotenv::from_filename("config.env");
    let passwd_result = var("MULTIFACTORY_PASSWD");
    let ip_result = var("MULTIFACTORY_SERVER_IP");

    // if there is no existing env file with a password and ip, create one
    if passwd_result.is_err() {
        stdout().write(String::from("Server Password: ").as_bytes());
        let mut passwd = String::new();

        // If the password can't be read, exit
        if stdin().read_line(&mut passwd).is_err() {
            println!("Failed to read Password!");
            exit(1);
        }

        stdout().write(String::from("Server IP: ").as_bytes());
        let mut ip = String::new();

        // If the ip can't be read, exit
        if stdin().read_line(&mut ip).is_err() {
            println!("Failed to read IP!");
            exit(1);
        }

        // Try to create the file
        let mut create_file_result = std::fs::File::create("config.env");

        // If the file can't be created, exit
        if create_file_result.is_err() {
            println!("Failed to create config!");
            exit(1);
        }

        let mut file = create_file_result.unwrap();

        // Try to write the data to the config file
        let config_str = String::from("MULTIFACTORY_PASSWD=") + passwd.as_str() + "\n" +
                               String::from("MULTIFACTORY_SERVER_IP=").as_str() + ip.as_str();

        // If the file can't be written to, exit
        if file.write_all(config_str.as_bytes()).is_err() {
            println!("Failed to write data to config!");
            exit(1);
        }

        return (passwd, ip);
    }

    // otherwise, just return the password
    return (passwd_result.unwrap(), ip_result.unwrap());
}

fn accept(to: &str) -> bool {
    full: String = to + " (y/n): ";
    stdout().write(full.as_bytes());
    let mut response = String::new();
    if stdin().read_line(&mut response).is_err() {
        return false;
    }

    return response == "Y" || response == "y";
}



#[tokio::main]
async fn main() {
    // Read Server IP and Password from Config
    let (passwd, ip) = prepare_env();

    // Create the TCP Stream
    let mut stream = TcpStream::connect(ip).await.unwrap();

    // Try logging in for 3 Times before quitting
    let mut login_failed = true;
    for i in 0..3 {
        let server_response: u8 = login(&passwd, &mut stream).await;
        // If the server accepted the login, continue
        if server_response == 0 || server_response == 1 {
            login_failed = false;
            break;
        }
        else if accept("Login Failed. Retry?") {
            continue;
        }
    }

    if login_failed {
        println!("Failed to login 3 times; Your IP has been excluded from the Server.");
        exit(1);
    }



}
