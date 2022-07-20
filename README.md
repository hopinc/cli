# Hop CLI

[![Build and release](https://github.com/hopinc/hop_cli/actions/workflows/release.yml/badge.svg)](https://github.com/hopinc/hop_cli/actions/workflows/release.yml)

The Hop CLI allows you to interface with Hop services through your command line. It can be used as a replacement for the [Console](https://console.hop.io/).

## Installation

During development, the Hop CLI can only be installed by directly compiling the binary on your machine. To do this, you'll first need to install [Rust](https://www.rust-lang.org/tools/install). Then, once you've cloned the repository, you can execute this command within the directory:

```bash
cargo install --path .
```

This will make the `hop` command available to you.

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
