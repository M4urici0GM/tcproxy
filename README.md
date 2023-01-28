# Tcproxy

<!--- These are examples. See https://shields.io for others or to customize this set of shields. You might want to include dependencies, project status and licence info here --->
![GitHub repo size](https://img.shields.io/github/repo-size/scottydocs/README-template.md)
![GitHub contributors](https://img.shields.io/github/contributors/scottydocs/README-template.md)
![GitHub stars](https://img.shields.io/github/stars/scottydocs/README-template.md?style=social)
![GitHub forks](https://img.shields.io/github/forks/scottydocs/README-template.md?style=social)

Project name is a tool that allows port forwarding to your local machine, even behind NAT.

## Prerequisites

Before you begin, ensure you have met the following requirements:
<!--- These are just example requirements. Add, duplicate or remove as required --->
- Rust 1.67 or later
- Mongodb instance (Optional if only server)

## Using Tcproxy Server

To use ```tcproxy-server```, follow these steps:

To see all options:
```
> tcproxy-server --help

With Cargo:
> cargo run --bin tcproxy-server -- --help
```

Starting the server listening on port 8080

```
> tcproxy --port 8080

With Cargo:
> cargo run --bin tcproxy-server -- --port 8080
```

Starting the server with a different port range available for proxy servers (default is from 15000-25000)
```
> tcproxy --port 8080

With Cargo:
> cargo run --bin tcproxy-server -- --port 8080
```




Add run commands and examples you think users will find useful. Provide an options reference for bonus points!

## Contributing to <project_name>
<!--- If your README is long or you have some specific process or steps you want contributors to follow, consider creating a separate CONTRIBUTING.md file--->
To contribute to <project_name>, follow these steps:

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