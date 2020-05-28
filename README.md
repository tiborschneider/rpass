# `rpass`

`rpass` is a password manager, based on [pass](https://www.passwordstore.org/), written in rust

## Why?

There are two main reasons why I have written `rpass`:

1. To add a simple UI, based on ROFI
2. The typical way for using [pass](https://www.passwordstore.org/) is to store every password as its own `.gpg` file. The Hierarchy represents the group, service and username: `[group]/[service]/[username].gpg`. Then, the idea is to synchronize this using `git`. Even though all passwords are encrypted, this already reveals too much information, like which services you use with which username, and how often you change the password. `rpass` solves this problem by storing all entries as encoded as `[uuid].gpg` (where the UUID is generated randomly, *not* on any personal data or the timestamp). Additionally, an `index.gpg` file keeps track of the uuids and the path where they would have been stored, for easy access.

## Structure

`rpass` stores all managed passwords in `~/.password-store/uuids`. The Index-file is stored at `~/.password-store/uuids/index.gpg`, and all keys are stored at `~/.password-store/uuids/[uuid].gpg`. `rpass` uses `pass` to manage all passwords. It is only an interface, to allow easy and comfortable access to the password files.

## Requirements & Installation

- [Rustup and Cargo](https://rustup.rs/)
- git
- ssh
- [xclip](https://github.com/astrand/xclip)
- [pass](https://www.passwordstore.org/)
- [rofi](https://github.com/davatorium/rofi)
- [fzf](https://github.com/junegunn/fzf)

On Arch Linux, install the requirements as:

```
sudo pacman -S git openssh pass rofi fzf
git clone https://github.com/tiborschneider/rpass.git
cd rpass
cargo install --path .
```

## Usage

### Migration / First Use

Before you can use `rpass`, you need to initialize the database. First, make sure that `pass` is setup and working. Ideally, add at least one password. Then, initialize `rpass` with 
```
rpass init
```
Then, choose which entries you wish to add to the rpass managed files. Any files not managed by `rpass` must reside outside of `~/.password-store/uuids`. They will never be touched by `rpass`.

### Regular Operation

```
USAGE:
    rpass [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    edit           Edit content of entry
    fix-index      Checks all indices and fixes them
    get            Print all entry information
    help           Prints this message or the help of the given subcommand(s)
    init           Initializes rpass and start the migration.
    insert         Insert a new key
    interactive    Copy username or password to clipboard using interactive dmenu
    ls             Lists all keys in a tree-like structure
    menu           Interactive app with rofi interface
    mv             Rename a specific key
    passwd         Change password of a specific key
    rm             Delete an existing key
    sync           Synchronize repository in non-uuid format. Without subcommand, sync local repos and start daemon.
```

### GUI operation

By running `rpass` without commands or flags, the main GUI application is started. By running `rpass interactive`, you can select an entry and copy the username, password or both to the clipboard. When copying both, `rpass` will first copy the username. Then, when calling `rpass interactive` the next time, it will copy the password. The username and the password will be kept in the clipboard for 5 seconds, after which, the clipboard will be cleared.

### Synchronization with mobile client

`rpass` allows you to have a separate repository at `~/.password-store/.sync/`, where the managed entries are stored in the regular format. This allows you to still use third party clients like a mobile client. However, you should not push this repository to a public server, like github. Instead, you should keep the remote **locally**, and synchronize with the mobile client while being in the same private network. Here is how you can set it up:

1. `rpass sync init` to initialize the second repository
2. Setup a git user named `git`, following the instructions here:
   https://git-scm.com/book/en/v2/Git-on-the-Server-Setting-Up-the-Server
   Make sure the user has a home directory, and setup the authorized keys properly. At least, add the key of your user and the one of the target device (android phone). Also, make sure that the shell is set to the git shell, and that it is working properly.
3. Login as git user:
   ```
   sh -s /bin/bash git
   ```
4. Generate an empty and raw repository
   ```
   mkdir rpass.git
   cd rpass.git
   git init --bare
   ```
5. logout of git user (exit)
6. make sure to add the git user to ssh AllowedUsers
   ```
   sudo vim /etc/ssh/sshd_conf
   ```
   Add git user to the AllowedUsers.
7. add the origin in the rpass folder:
   ```
   cd ~/.password-store/.sync
   git remote add origin ssh://git@localhost/~git/rpass.git
   ```
8. Somehow get the gpg key to the mobile device

Now, the synchronization is setup. To start the synchronization, run
```
rpass sync
```
While the ssh daemon is running, and your mobile phone is in the same local network, you can synchronize the repository. The synchronization works in both ways; changes done in `rpass` and changes done on the local device will both be applied.

## Licence

`rpass` was written by Tibor Schneider and is licensed under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html) licence.
