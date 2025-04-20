use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Write, BufReader, BufRead, Read};
use std::fs::OpenOptions;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap; 

fn handle_client_ssh(mut stream: TcpStream) {
   buffer.clear();
    reader.read_line(&mut buffer).expect("Erreur lecture mdp");
    let password = buffer.trim().to_string();

    if login != "admin" || password != "admin-techpro" {
        stream.write_all(b"Permission denied, please try again.\r\n").unwrap();
        println!("[-] Identifiants incorrects pour {}/{}", login, password);
        return; 
    }

    stream.write_all(b"\r\nWelcome to Ubuntu 20.04.6 LTS (GNU/Linux 5.4.0-42-generic x86_64)\r\n\r\n").unwrap();
    stream.flush().unwrap();
    println!("[+] Authentification réussie pour {}", login);
}

fn main() -> std::io::Result<()> {
    println!("[*] Démarrage du honeypot SSH...");
    
    Ok(())
}

