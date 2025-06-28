# Honeypot SSH en Rust

Un honeypot SSH simple et personnalisable, développé en Rust. Ce projet simule un serveur SSH rudimentaire, conçu pour capturer les tentatives de connexion et les interactions de commande de potentiels attaquants, en enregistrant toute l'activité pour une analyse ultérieure.

---

## Prérequis

Avant de pouvoir exécuter ce honeypot, vous aurez besoin de :

* **Rust et Cargo :** Assurez-vous d'avoir la chaîne d'outils Rust installée. Si ce n'est pas le cas, vous pouvez l'installer via `rustup` :
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
    ```

---

## Fonctionnalités

* **Interaction de type SSH :** Imite une invite de connexion SSH basique et une interface de ligne de commande.
* **Capture d'Identifiants :** Enregistre tous les noms d'utilisateur et mots de passe tentés, qu'ils soient corrects ou non.
* **Journalisation des Commandes :** Enregistre chaque commande exécutée par le client connecté.
* **Simulation de Répertoire :** Répond aux commandes `cd` et `ls` avec des structures de répertoire prédéfinies.
* **Simulation de Contenu de Fichier :** Fournit du contenu fictif pour des commandes comme `cat /etc/passwd` ou `cat /tmp/backup.sh`.
* **Invites Personnalisables :** Offre des invites de shell réalistes (`user@ubuntu:~ $`).
* **Multi-threadé :** Gère plusieurs connexions entrantes simultanément.

---

## Aperçu des Fonctionnalités

Votre honeypot SSH est structuré autour de deux fonctions principales :

### 1. `fn handle_client_ssh(mut stream: TcpStream)`

Cette fonction gère la **session simulée pour chaque client connecté**. Ses responsabilités incluent :

* **Authentification :** Simule le processus de login SSH, gérant les identifiants corrects (`admin`/`admin-techpro`) et incorrects.
* **Journalisation :** Enregistre toutes les tentatives de connexion (réussies/échouées) et chaque commande exécutée par le client dans `honeypot.log`.
* **Shell Simulé :** Fournit un pseudo-shell interactif, affichant des invites de commande réalistes et répondant à un ensemble prédéfini de commandes Linux (`ls`, `cd`, `cat`, `wget`, `sudo`, etc.) avec des sorties et des structures de répertoire simulées.
* **Gestion de Session :** Ferme la connexion en cas de commande `exit`/`logout` ou de déconnexion du client.

### 2. `fn main() -> std::io::Result<()>`

Cette fonction est le **point d'entrée principal** de l'application honeypot. Elle se charge de :

* **Démarrage du Serveur :** Lance le honeypot et le lie à l'adresse `0.0.0.0` sur le port `23` (port Telnet par défaut, utilisé ici pour la simulation SSH).
* **Écoute des Connexions :** Crée un thread dédié pour écouter en continu les connexions entrantes.
* **Gestion Concurrente :** Pour chaque nouveau client qui se connecte, elle **crée un nouveau thread** et lui confie la gestion de la session via `handle_client_ssh`, permettant au honeypot de gérer plusieurs interactions simultanément.
* **Maintien du Serveur :** Assure que le honeypot reste actif et continue d'écouter les nouvelles connexions indéfiniment.

---

## Commandes Simulées

Après une connexion réussie, le honeypot fournit des réponses simulées pour une série de commandes Linux courantes, notamment :

* `ls`, `ls -al`, `ls -la`
* `cd <répertoire>` (par exemple, `cd /tmp`, `cd ~`)
* `pwd`
* `hostname`, `whoami`, `id`, `ps`
* `cat /etc/passwd`, `cat /etc/group`, `cat /etc/hosts`, `cat ~/.ssh/authorized_keys`
* `cat backup.sh` (si vous êtes dans le répertoire `/tmp`)
* `sudo <commande>` (simule une demande de mot de passe et un refus)
* `wget <url>`, `curl <url>` (téléchargements simulés)
* `scp <source> <cible>` (transfert de fichiers simulé)
* `uname -a`
* `exit`, `logout`

Toute commande non reconnue entraînera un message "command not found".

---

## Journalisation (Logging)

Toutes les interactions significatives avec le honeypot sont enregistrées dans un fichier nommé `honeypot.log` situé à la racine de votre projet. Cela inclut :

* Les tentatives de connexion du client.
* Les tentatives de connexion incorrectes (nom d'utilisateur et mot de passe utilisés, ainsi que l'adresse IP source).
* Chaque commande exécutée par un client connecté (avec l'adresse IP source et l'horodatage).

Le fichier journal est créé s'il n'existe pas et ajoute de nouvelles entrées, assurant un enregistrement continu de l'activité.
 
