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
    
    loop {
        let prompt = format!("user@ubuntu:{}$ ", current_dir);
        stream.write_all(prompt.as_bytes()).unwrap();
        stream.flush().unwrap();
        
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("[-] Le client a fermé la connexion.");
                break;
            }
            Ok(_) => {
                let input = buffer.trim();

                match input {
                    "ls" => {
                        if current_dir == "~" {
                            stream.write_all(b"Desktop  Documents Downloads Music Pictures  Public Templates  Videos\r\n").unwrap();
                        } else if directories.contains_key(current_dir.as_str()) {
                            let files = directories.get(current_dir.as_str()).unwrap();
                            if files.is_empty() {
                                stream.write_all(b"\r\n").unwrap();
                            } else {
                                let content = files.join("  ");
                                stream.write_all(format!("{}\r\n", content).as_bytes()).unwrap();
                            }
                        } else {
                            stream.write_all(b"Desktop  Documents Downloads Music Pictures  Public Templates  Videos\r\n").unwrap();
                        }
                    }
                    "ls -al" | "ls -la" => {
                        if current_dir == "/tmp" {
                            stream.write_all(b"total 12\r\n").unwrap();
                            stream.write_all(b"drwxrwxrwt  3 root root 4096 Jan 15 10:30 .\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 20 root root 4096 Jan 15 09:15 ..\r\n").unwrap();
                            stream.write_all(b"-rwxrwxrwx  1 user user 1337 Jan 15 10:29 backup.sh.txt\r\n").unwrap();
                        } else if current_dir == "~" {
                            stream.write_all(b"total 32\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 9 user user 4096 Jan 15 10:30 .\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 3 root root 4096 Jan 15 09:15 ..\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Desktop\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Documents\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Downloads\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Music\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Pictures\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Public\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Templates\r\n").unwrap();
                            stream.write_all(b"drwxr-xr-x 2 user user 4096 Jan 15 10:00 Videos\r\n").unwrap();
                        } else {
                            stream.write_all(b"total 0\r\n").unwrap();
                        }
                    }
                    "hostname" => {
                        stream.write_all(b"ubuntu\r\n").unwrap();
                    }
                    "exit" | "logout" => {
                        stream.write_all(b"logout\r\n").unwrap();
                        break;
                    }
                    "" => {

                    }
                    _ => {
                        stream.write_all(format!("{}: command not found\r\n", input).as_bytes()).unwrap();
                    }
                }
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("[!] Erreur de lecture : {}", e);
                break;
            }
        }
    }
    
    println!("[-] Connexion fermée pour {}", stream.peer_addr().unwrap());

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
