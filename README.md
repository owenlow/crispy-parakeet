# ORW - Owen's Rust Webserver

### Summary

Hack project to learn about Rust. Hosts pages as template files in the `/pages` directory.

### Usage

First, install dependencies:
```shell
$ cargo build 
```

Then start the server:

```shell
$ cargo run
```

The server will now be running at `http://localhost:7878/`, reading from `src/pages/index.hbs`.

### TODO

* Add some sort of controller support. It was out of scope for the hackathon.
* Safer defaults and path/query handling
* Make multithreaded: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
* Support request types other than GET
