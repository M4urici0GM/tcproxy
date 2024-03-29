# Tcproxy

Project name is a tool that allows port forwarding to your local machine, even behind firewall.

## Prerequisites

Before you begin, ensure you have met the following requirements:
- Rust 1.67 or later

## How it works
This project uses multiplexing for redirecting all traffic from an public and exposed port
to a local port in your machine, using a single tcp connection between server and your machine.

The following diagram shows how the project would work when the server receives 3 connections, and redirects the traffic to the local machine, demuxing 
the connections from the server.

![diagram.png](docs/imgs/diagram.png)

## Running it locally (dev environment)
1. Fork the repository
2. Start the server with ```cargo run --bin tcproxy-server -- --port 8080```
3. In another terminal, create a new app context with ```cargo run --bin tcproxy-cli -- context create main 127.0.0.1:8080```
4. Then Start the client with ```cargo run --bin tcproxy-cli -- listen 3338```

You should be able to see something like this on your terminal:
![terminal-screenshot.png](docs/imgs/terminal-screenshot.png)

So now, every traffic sent to 127.0.0.1:21017 is going to be redirected
to 127.0.0.1:3338

## Using Tcproxy Server

To spawn  ```tcproxy-server```, follow these steps:

To see all options:
```
$ tcproxy-server --help
```

Starting the server listening on port 8080

```
$ tcproxy --port 8080
```

Starting the server with a different port range available for proxy servers (default is from 15000-25000)
```
$ tcproxy --port 8080
```

## Using Tcproxy Client (cli)

To see all options:
```
$ tcproxy-cli --help
```

Starting to receive remote connections:
```
$ tcproxy-cli listen <local-port>
```

Starting to receive remote connections using an app context:
```
$ tcproxy-cli listen <local-port> --app-context <name>
```

### App Contexts
Contexts are like origins on git, you can have multiple ones, and when starting to listen,
you can specify to where tcproxy-cli is going to connect. By default tcproxy-cli doesnt
contain any app context, and when you create the first one, it is set as the default.

Listing all your available contexts:
```
$ tcproxy-cli context list
```

Creating new context
```
$ tcproxy-cli context create <name> <ip>:<port>
```

Setting an existing context as default:
```
$ tcproxy-cli context set-default <name>
```

## Contributing to tcproxy
To contribute to this project, follow these steps:

1. Fork this repository.
2. Create a branch: `git checkout -b <branch_name>`.
3. Make your changes and commit them: `git commit -m '<commit_message>'`
4. Push to the original branch: `git push origin <project_name>/<location>`
5. Create the pull request.

Alternatively see the GitHub documentation on [creating a pull request](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request).

## Contributors

Thanks to the following people who have contributed to this project:

* [@m4urici0gm](https://github.com/m4urici0gm) 📖

## Contact

If you want to contact me you can reach me at contact@mgbarbosa.dev

## License
<!--- If you're not sure which open license to use see https://choosealicense.com/--->

This project uses the following license: [GPL-2.0](https://github.com/M4urici0GM/tcproxy/blob/main/LICENSE.md).

Done with :heart: