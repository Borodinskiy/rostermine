# 💎⛏️ Rostermine 💎⛏️
Minecraft launcher, which primary goal is to achieve quieter startup for your favorite minecraft versions / modpacks / snapshots / etc.

## Features
- Simple argumets for game launch, that suitable for shortcut creation
```sh
# launches latest release
$ rostermine
# you can select versions
$ rostermine -l 1.16.5
# and choose instance dir
$ rostermine --launch 1.18.2 -i instances/cavescliffs
$ rostermine --launch 1.18.2 --instance-dir instances/cavescliffs
```
- Ability to change game instance directory allows to easily switch between modpacks configurations
- Offline mode support

## TODOs
- Implement Online authorisation
- Modloaders support
- Fix game versions before 1.13 on linux hosts (they don't launch)
- Configuration file for version aliases. Example syntax:
```yaml
# ~/.rostermine/config.yaml
version: 1.0
instances_directory: /home/bebra/games/minecraft
aliases:
  SevTech Ages:
    version: 1.12.2
    forge: 1.543.32421312
    directory: sevtech-ages
  1.12.2:
  1.12.2-testlings:
    version: 1.12.2
    directory: 1.12.2-testling
java:
  v8:
    path: /usr/lib/jvm/openjdk8
  v17:
    path: /usr/lib/jvm/openjdk17
  default:
    path: /usr/lib/jvm/openjdk
```
- ...
- GUI?

# Building

## Windows
1. Install rust via rustup or other tools
2. Execute inside project directory
```sh
$ cargo build --release
```
3. Binary goes into target/release directory

## Linux

1. Uhm... Hmhmh... docker compose :3
```
# docker compose up
```
2. Binary goes into target-docker/release directory. Should work on most distributions without pain... Install java

## Macos
Idk.