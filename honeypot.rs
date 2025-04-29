use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Write, BufReader, BufRead, Read};
use std::fs::OpenOptions;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap; 

fn handle_client_ssh(mut stream: TcpStream) {
    println!("\n[+] Connexion SSH !");
    println!("Client: {}", stream.peer_addr().unwrap());

    stream.write_all(b"login as: ").expect("Erreur d'écriture");
    stream.flush().unwrap();

    let mut reader = BufReader::new(stream.try_clone().expect("Erreur clone stream"));
    let mut buffer = String::new();

    buffer.clear();
    reader.read_line(&mut buffer).expect("Erreur lecture login");
    let login = buffer.trim().to_string();

    stream.write_all(format!("{}@ubuntu's password: ", login).as_bytes()).expect("Erreur d'écriture");
    stream.flush().unwrap();
    
    buffer.clear();
    reader.read_line(&mut buffer).expect("Erreur lecture mdp");
    let password = buffer.trim().to_string();

    if login != "admin" || password != "admin-techpro" {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        stream.write_all(b"Permission denied, please try again.\r\n").unwrap();
        println!("[-] Identifiants incorrects pour {}/{}", login, password);
        writeln!(log_file, "[{}] Identifiants incorrects pour {}/{} de l'ip {}",now.as_secs(),login, password, stream.peer_addr().unwrap()).expect("Erreur d'écriture dans le fichier log");
        return;
    }

    stream.write_all(b"\r\nWelcome to Ubuntu 20.04.6 LTS (GNU/Linux 5.4.0-42-generic x86_64)\r\n\r\n").unwrap();
    stream.flush().unwrap();
    println!("[+] Authentification réussie pour {}", login);


    let mut current_dir = "~".to_string();

    let mut directories: HashMap<&str, Vec<&str>> = HashMap::new();
    directories.insert("/Desktop", vec![]);
    directories.insert("/Downloads", vec![]);
    directories.insert("/Documents", vec![]);
    directories.insert("/tmp", vec!["backup.sh"]);
    directories.insert("/home", vec!["/user"]);
    directories.insert("user", vec!["Desktop  Documents Downloads Music Pictures  Public Templates  Videos"]);
    directories.insert("/var", vec![]);
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
