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
                    cmd if cmd.starts_with("cd ") => {
                        let parts: Vec<&str> = cmd.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let target = parts[1];
                            if directories.contains_key(target) {
                                current_dir = target.to_string();
                            } else {
                                stream.write_all(format!("bash: cd: {}: No such file or directory\r\n", target).as_bytes()).unwrap();
                            }
                        } else {
                            current_dir = "~".to_string();
                        }
                    }
                    cmd if cmd.contains("wget") => {
                        let parts: Vec<&str> = cmd.split_whitespace().collect();
                        
                        if parts.len() >= 2 {
                            let url = parts[1];
                            stream.write_all(format!("Download {} done                                    100% 1337     1.2MB/s   00:00\r\n",url).as_bytes()).unwrap();
                        } else {
                            stream.write_all(b"usage: wget [url]\r\n").unwrap();
                        }
                    }
                    cmd if cmd.contains("curl") => {
                        stream.write_all(b"  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current\r\n").unwrap();
                        stream.write_all(b"                                 Dload  Upload   Total   Spent    Left  Speed\r\n").unwrap();
                        stream.write_all(b"100  1337  100  1337    0     0   2.1M      0 --:--:-- --:--:-- --:--:--  2.1M\r\n").unwrap();
                    }
                    cmd if cmd.contains("scp") => {
                        let parts: Vec<&str> = cmd.split_whitespace().collect();
                        
                        if parts.len() >= 3 {
                            let file = parts[1];
                            let target = parts[2];
                            stream.write_all(format!(
                                "{}                                    100% 1337     1.2MB/s   00:00\r\n",
                                format!("{}:{}", target, file)
                            ).as_bytes()).unwrap();
                        } else {
                            stream.write_all(b"usage: scp [source] [target]\r\n").unwrap();
                        }
                    }
                    "pwd" => {
                        if current_dir == "~" {
                            stream.write_all(b"/home/user\r\n").unwrap();
                        } else {
                            stream.write_all(format!("{}\r\n", current_dir).as_bytes()).unwrap();
                        }
                    }
                    "id" => {
                        stream.write_all(b"uid=1000(user) gid=1000(user) groups=1000(user)\r\n").unwrap();
                    }
                    cmd if cmd.starts_with("sudo") => {
                        stream.write_all(b"[sudo] password for user: ").unwrap();
                        stream.flush().unwrap();

                        let mut pw_buffer = Vec::new();
                        let mut reader = std::io::BufReader::new(&stream);
                        use std::io::BufRead;

                        match reader.read_until(b'\n', &mut pw_buffer) {
                            Ok(_) => {
                                stream.write_all(b"Sorry, try again.\r\n").unwrap();
                            }
                            Err(_) => {
                                stream.write_all(b"\r\n").unwrap();
                            }
                        }
                    }
                    "whoami" => {
                        stream.write_all(b"user\r\n").unwrap();
                    }
                    "ps" => {
                        stream.write_all(b"  PID TTY          TIME CMD\r\n").unwrap();
                        stream.write_all(b" 2809 pts/0    00:00:00 bash\r\n").unwrap();
                        stream.write_all(b" 3737 pts/0    00:00:00 ps\r\n").unwrap();
                    }
                    "cat /etc/passwd" => {
                        stream.write_all(b"root:x:0:0:root:/root:/bin/bash\r\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\r\nbin:x:2:2:bin:/bin:/usr/sbin/nologin\r\nsys:x:3:3:sys:/dev:/usr/sbin/nologin\r\nsync:x:4:65534:sync:/bin:/bin/sync\r\ngames:x:5:60:games:/usr/games:/usr/sbin/nologin\r\nman:x:6:12:man:/var/cache/man:/usr/sbin/nologin\r\nlp:x:7:7:lp:/var/spool/lpd:/usr/sbin/nologin\r\nmail:x:8:8:mail:/var/mail:/usr/sbin/nologin\r\nnews:x:9:9:news:/var/spool/news:/usr/sbin/nologin\r\nuucp:x:10:10:uucp:/var/spool/uucp:/usr/sbin/nologin\r\nproxy:x:13:13:proxy:/bin:/usr/sbin/nologin\r\nwww-data:x:33:33:www-data:/var/www:/usr/sbin/nologin\r\nbackup:x:34:34:backup:/var/backups:/usr/sbin/nologin\r\nlist:x:38:38:Mailing List Manager:/var/list:/usr/sbin/nologin\r\nirc:x:39:39:ircd:/var/run/ircd:/usr/sbin/nologin\r\ngnats:x:41:41:Gnats Bug-Reporting System (admin):/var/lib/gnats:/usr/sbin/nologin\r\nnobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\r\nsystemd-network:x:100:102:systemd Network Management,,,:/run/systemd:/usr/sbin/nologin\r\nsystemd-resolve:x:101:103:systemd Resolver,,,:/run/systemd:/usr/sbin/nologin\r\nsystemd-timesync:x:102:104:systemd Time Synchronization,,,:/run/systemd:/usr/sbin/nologin\r\nmessagebus:x:103:106::/nonexistent:/usr/sbin/nologin\r\nsyslog:x:104:110::/home/syslog:/usr/sbin/nologin\r\n_apt:x:105:65534::/nonexistent:/usr/sbin/nologin\r\ntss:x:106:111:TPM software stack,,,:/var/lib/tpm:/bin/false\r\nuuidd:x:107:114::/run/uuidd:/usr/sbin/nologin\r\ntcpdump:x:108:115::/nonexistent:/usr/sbin/nologin\r\navahi-autoipd:x:109:116:Avahi autoip daemon,,,:/var/lib/avahi-autoipd:/usr/sbin/nologin\r\nusbmux:x:110:46:usbmux daemon,,,:/var/lib/usbmux:/usr/sbin/nologin\r\nrtkit:x:111:117:RealtimeKit,,,:/proc:/usr/sbin/nologin\r\ndnsmasq:x:112:65534:dnsmasq,,,:/var/lib/misc:/usr/sbin/nologin\r\ncups-pk-helper:x:113:120:user for cups-pk-helper service,,,:/home/cups-pk-helper:/usr/sbin/nologin\r\nspeech-dispatcher:x:114:29:Speech Dispatcher,,,:/run/speech-dispatcher:/bin/false\r\navahi:x:115:121:Avahi mDNS daemon,,,:/var/run/avahi-daemon:/usr/sbin/nologin\r\nkernoops:x:116:65534:Kernel Oops Tracking Daemon,,,:/:/usr/sbin/nologin\r\nsaned:x:117:123::/var/lib/saned:/usr/sbin/nologin\r\nnm-openvpn:x:118:124:NetworkManager OpenVPN,,,:/var/lib/openvpn/chroot:/usr/sbin/nologin\r\nhplip:x:119:7:HPLIP system user,,,:/run/hplip:/bin/false\r\nwhoopsie:x:120:125::/nonexistent:/bin/false\r\ncolord:x:121:126:colord colour management daemon,,,:/var/lib/colord:/usr/sbin/nologin\r\nfwupd-refresh:x:122:127:fwupd-refresh user,,,:/run/systemd:/usr/sbin/nologin\r\ngeoclue:x:123:128::/var/lib/geoclue:/usr/sbin/nologin\r\npulse:x:124:129:PulseAudio daemon,,,:/var/run/pulse:/usr/sbin/nologin\r\ngnome-initial-setup:x:125:65534::/run/gnome-initial-setup/:/bin/false\r\ngdm:x:126:131:Gnome Display Manager:/var/lib/gdm3:/bin/false\r\nsssd:x:127:132:SSSD system user,,,:/var/lib/sss:/usr/sbin/nologin\r\nuser:x:1000:1000:Ubuntu test,,,:/home/user:/bin/bash\r\nsystemd-coredump:x:999:999:systemd Core Dumper:/:/usr/sbin/nologin\r\nepmd:x:128:136::/var/run/epmd:/usr/sbin/nologin\r\nsshd:x:129:65534::/run/sshd:/usr/sbin/nologin\r\n").unwrap();
                    }
                    "cat /etc/group" => {
                        stream.write_all(b"root:x:0:\r\ndaemon:x:1:\r\nbin:x:2:\r\nsys:x:3:\r\nadm:x:4:syslog,user\r\ntty:x:5:syslog\r\ndisk:x:6:\r\nlp:x:7:\r\nmail:x:8:\r\nnews:x:9:\r\nuucp:x:10:\r\nman:x:12:\r\nproxy:x:13:\r\nkmem:x:15:\r\ndialout:x:20:\r\nfax:x:21:\r\nvoice:x:22:\r\ncdrom:x:24:user\r\nfloppy:x:25:\r\ntape:x:26:\r\nsudo:x:27:user\r\naudio:x:29:pulse\r\ndip:x:30:user\r\nwww-data:x:33:\r\nbackup:x:34:\r\noperator:x:37:\r\nlist:x:38:\r\nirc:x:39:\r\nsrc:x:40:\r\ngnats:x:41:\r\nshadow:x:42:\r\nutmp:x:43:\r\nvideo:x:44:\r\nsasl:x:45:\r\nplugdev:x:46:user\r\nstaff:x:50:\r\ngames:x:60:\r\nusers:x:100:\r\nnogroup:x:65534:\r\nsystemd-journal:x:101:\r\nsystemd-network:x:102:\r\nsystemd-resolve:x:103:\r\nsystemd-timesync:x:104:\r\ncrontab:x:105:\r\nmessagebus:x:106:\r\ninput:x:107:\r\nkvm:x:108:\r\nrender:x:109:\r\nsyslog:x:110:\r\ntss:x:111:\r\nbluetooth:x:112:\r\nssl-cert:x:113:\r\nuuidd:x:114:\r\ntcpdump:x:115:\r\navahi-autoipd:x:116:\r\nrtkit:x:117:\r\nssh:x:118:\r\nnetdev:x:119:\r\nlpadmin:x:120:user\r\navahi:x:121:\r\nscanner:x:122:saned\r\nsaned:x:123:\r\nnm-openvpn:x:124:\r\nwhoopsie:x:125:\r\ncolord:x:126:\r\nfwupd-refresh:x:127:\r\ngeoclue:x:128:\r\npulse:x:129:\r\npulse-access:x:130:\r\ngdm:x:131:\r\nsssd:x:132:\r\nlxd:x:133:user\r\nuser:x:1000:\r\nsambashare:x:134:user\r\nsystemd-coredump:x:999:\r\nrdma:x:135:\r\nepmd:x:136:\r\n").unwrap();
                    }
                    "cat ~/.ssh/authorized_keys" => {
                        stream.write_all(b"ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEAvkY9vDBt8nGp7L5MiulbOY2DBRrF2JjT3vAaF4e3y2jRgoJmCFX/7QzBcIYUpbfUkuzf+qQ9UXE8F5L6uwU7CbWZft9fM8z8c3n8kgzGw6yQ7Q1vAkHVJ5HZk5h8VtfTYGuRhD1EfWuv0GqYxl7YO+Wv5nYm0wGZrZD1XzDhQ== user@ubuntu\r\n").unwrap();
                    }
                    "cat ~/.ssh/id_rsa" => {
                        stream.write_all(b"cat: /home/user/.ssh/id_rsa: Permission denied\r\n").unwrap();
                    }
                    "cat /etc/hosts" => {
                        stream.write_all(b"127.0.0.1\tlocalhost\r\n127.0.1.1\tubuntu\r\n").unwrap();
                    }
                    "cat /etc/sudoers" => {
                        stream.write_all(b"cat: /etc/sudoers: Permission denied\r\n").unwrap();
                    }
                    "uname -a" => {
                        stream.write_all(b"Linux ubuntu 5.4.0-42-generic #46-Ubuntu SMP Fri Jul 10 00:24:02 UTC 2020 x86_64 x86_64 x86_64 GNU/Linux\r\n").unwrap();
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
