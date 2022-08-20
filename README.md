# Hop CLI

[![Build and release](https://github.com/hopinc/hop_cli/actions/workflows/release.yml/badge.svg)](https://github.com/hopinc/hop_cli/actions/workflows/release.yml)

The Hop CLI allows you to interface with Hop services through your command line. It can be used as a replacement for the [Console](https://console.hop.io/).

## Installation

> Any of the following will make the `hop` command available to you.

### Linux, MacOS and FreeBSD

It can be installed with our universal install script:

```bash
$ curl -fsSL https://download.hop.sh/install | sh
```

### Arch Linux

Use your favourite AUR helper to install the package:

```bash
# yay example
$ yay -S hop-cli
```


### Windows

Install with the [Hop Windows Installer 64bit](https://download.hop.sh/windows/x86_64) or the [Hop Windows Installer 32bit](https://download.hop.sh/windows/i686)

### Source

The Hop CLI can only be installed by directly compiling the binary on your machine. To do this, you'll first need to install [Rust](https://www.rust-lang.org/tools/install). Then, once you've cloned the repository, you can execute this command within the directory:

```bash
cargo install --path .
```

## Logging In

To get started, you need to login to your Hop account through the CLI:

```bash
hop auth login
```

A browser window will open the Hop Console and prompt you to allow the CLI to connect to your account. Once done, you will be redirected back.

That's all! You can now start using the CLI.

## Usage

### Projects

You can set a default project to use which will automatically be applied to every command.

```bash
hop projects switch
```

You can override it by passing the `--project` argument. For example: `hop deploy --project api`.

### Deploying

To deploy a project directory, first navigate to the directory through `cd` and then execute:

```bash
hop deploy
```

This will deploy the project to Hop, or create a new one if you don't have a Hopfile (`hop.yml`) already.

### Linking

To link a project to a service, first navigate to the directory through `cd` and then execute:

```bash
hop link
```

This will link the directory to the deployment and create a Hopfile (`hop.yml`).

## Contributing

Contributions are welcome! Please open an issue or pull request if you find any bugs or have any suggestions.
