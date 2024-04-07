use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Write, BufReader, BufRead, Read};
use std::fs::OpenOptions;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap; 

fn handle_client_ssh(mut stream: TcpStream) {
   
}

fn main() -> std::io::Result<()> {
    println!("[*] Démarrage du honeypot SSH...");
    let ssh_thread = thread::spawn(|| {
        let listener = TcpListener::bind("0.0.0.0:23").expect("Erreur bind port 23");
        println!("[] Honeypot en écoute sur 0.0.0.0:23");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_client_ssh(stream);
                    });
                }
                Err(e) => {
                    println!("[!] Erreur connexion: {}", e);
                }
            }
        }
    });

    ssh_thread.join().unwrap();
    Ok(())
}
