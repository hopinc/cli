# Hop CLI

The Hop CLI allows you to interface with Hop services through your command line. It can be used as a replacement for the Console.

## Installing

During development, the Hop CLI can only be installed by directly compiling the binary on your machine. To do this, you'll first need to install [Rust](https://rust-lang.org). Then, once you've cloned the repository, you can execute this command within the directory:

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

That's all! You can now start using the CLi.

## Usage

### Projects

You can set a default project to use which will automatically be applied to every command.

```bash
hop projects switch
```

Even when you have a default set, however, you can override it by passing the `--project` argument to a command with the project's namespace. For example: `hop deploy --project api`.

### Deploying

To deploy a project directory, first navigate to the directory through `cd` and then execute:

```bash
hop deploy
```

This will walk you through the steps needed to create a deployment, unless you already have a `hop.json` within the current directory. Then, the directory will be uploaded to our build servers and deployed to Hop!
