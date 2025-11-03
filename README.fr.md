# Dedupler

`dedupler` est un outil en ligne de commande simple et rapide écrit en Rust pour supprimer les lignes en double des fichiers. Il peut traiter un seul fichier ou un répertoire entier, avec des options pour écrire la sortie dans un fichier ou dans le terminal, ignorer des fichiers spécifiques et afficher des statistiques d'exécution.

## Fonctionnalités

- Dédoublonnage rapide grâce à `HashSet` pour un traitement efficace.
- Traitement récursif des répertoires pour trouver et gérer les fichiers.
- Prise en charge des motifs `.gitignore` et des règles d'ignorance personnalisées avec `ignore`.
- Sortie flexible vers un fichier spécifié ou la sortie standard.
- Retour visuel avec une barre de progression `indicatif`.
- Statistiques d'exécution détaillées (lignes lues, doublons, temps de traitement).
- Compatibilité multiplateforme avec Linux, macOS et Windows.
- Gestion robuste de divers encodages de fichiers (UTF-8, UTF-16, Windows-1252).

## Dépendances

Ce projet utilise les dépendances suivantes (telles que définies dans `Cargo.toml`) :

- `clap` (version `4.5.51`) : Pour l'analyse des arguments de la ligne de commande.
- `indicatif` (version `0.18.2`) : Pour afficher une barre de progression.
- `encoding_rs` (version `0.8.35`) : Pour la gestion des encodages de fichiers.
- `encoding_rs_io` (version `0.1.7`) : Pour la lecture de fichiers avec différents encodages.
- `ignore` (version `0.4.25`) : Pour ignorer les fichiers et répertoires.
- `tempfile` (version `3.23.0`) : Pour la création de fichiers et répertoires temporaires dans les tests.

## Installation

### Prérequis

Assurez-vous d'avoir Rust et Cargo d'installés sur votre système. Vous pouvez les installer en suivant les instructions sur le site officiel de Rust : [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compilation pour Linux (depuis Linux)
1.  Clonez ce dépôt :
    ```sh
    git clone https://github.com/cederig/dedupler.git
    cd dedupler
    ```
2.  Compilez le projet :
    ```sh
    cargo build --release
    ```
    L'exécutable se trouvera dans `target/release/dedupler`.

### Compilation pour macOS (depuis Linux/macOS)

Pour compiler ce projet pour Windows à partir d'un autre système d'exploitation (comme Linux ou macOS), vous pouvez utiliser la compilation croisée. Vous aurez besoin de la cible Rust pour Windows.

1.  Ajoutez la cible Windows à votre installation Rust :
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2.  Compilez le projet pour la cible Windows :
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

L'exécutable pour Windows se trouvera dans `target/x86_64-pc-windows-gnu/release/dedupler.exe`.

### Compilation pour macOS (depuis Linux/macOS)

Pour compiler ce projet pour macOS à partir d'un autre système d'exploitation (comme Linux ou macOS), vous pouvez utiliser la compilation croisée. Vous aurez besoin de la cible Rust pour macOS.

1.  Ajoutez la cible macOS à votre installation Rust (choisissez la bonne architecture) :
    *   Pour les Mac Intel (x86_64) :
        ```sh
        rustup target add x86_64-apple-darwin
        ```
    *   Pour les Mac Apple Silicon (aarch64) :
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2.  Compilez le projet pour la cible macOS (choisissez la bonne architecture) :
    *   Pour les Mac Intel :
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
    *   Pour les Mac Apple Silicon :
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

L'exécutable pour macOS se trouvera dans `target/<votre_cible_mac>/release/dedupler`.

## Utilisation

```sh
dedupler [OPTIONS] [FILE]
```

### Arguments

-   `[FILE]` : Le fichier d'entrée à traiter. Ne peut pas être utilisé avec `-d` / `--directory`.

### Options

-   `-d, --directory <DIRECTORY>` : Traite tous les fichiers dans le répertoire spécifié. Ne peut pas être utilisé avec `[FILE]`.
-   `-o, --output <OUTPUT>` : Chemin vers le fichier de sortie. Si non fourni, les résultats sont affichés dans le terminal. Lors du traitement d'un répertoire, cela spécifie un répertoire de sortie pour refléter la structure d'entrée.
-   `--stat` : Affiche les statistiques d'exécution détaillées.
-   `--ignore <PATTERN>` : Un motif glob de fichiers/répertoires à ignorer. Peut être spécifié plusieurs fois. (par exemple, `--ignore '*.log' --ignore 'tmp/'`)
-   `-h, --help` : Affiche les informations d'aide.
-   `-V, --version` : Affiche les informations de version.

## Exemples

-   Dédoublonner un seul fichier et afficher dans le terminal :
    ```sh
    ./dedupler mon_fichier.txt
    ```

-   Dédoublonner un fichier et enregistrer dans un autre fichier :
    ```sh
    ./dedupler mon_fichier.txt -o mon_fichier_dedoublonne.txt
    ```

-   Dédoublonner un fichier et afficher les statistiques :
    ```sh
    ./dedupler mon_fichier.txt --stat
    ```

-   Dédoublonner tous les fichiers d'un répertoire et les enregistrer dans un nouveau répertoire :
    ```sh
    mkdir repertoire_sortie
    ./dedupler -d ./repertoire_source -o ./repertoire_sortie
    ```

-   Dédoublonner un répertoire, en ignorant les fichiers de log et le sous-répertoire `temp`:
    ```sh
    ./dedupler -d ./mon_projet --ignore '*.log' --ignore 'temp/'
    ```

## Ignorer des Fichiers

`dedupler` respecte automatiquement les règles définies dans les fichiers `.gitignore` et `.ignore` dans le répertoire en cours de traitement.

Vous pouvez ajouter d'autres motifs d'ignorance à l'aide de l'option `--ignore`.

Par exemple :
-   `--ignore '*.tmp'`: Ignore tous les fichiers avec l'extension `.tmp`.
-   `--ignore 'logs/'`: Ignore le répertoire `logs`.
-   `--ignore '**/temp*'`: Ignore tous les fichiers et répertoires commençant par `temp` dans n'importe quel sous-répertoire.

## Tests

Ce projet inclut des tests unitaires; pour les exécuter, utilisez la commande suivante à la racine du projet :

```sh
cargo test
```

Cette commande compile le programme en mode test et exécute toutes les fonctions de test.