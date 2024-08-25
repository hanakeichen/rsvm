# Introduction

This project is a hobby JVM. The main purpose is to learn the JVM and implement a subset of JVM 7 using Rust.

# Requirements

- x86-64(Windows/Linux/MacOS)
- Git
- Cargo
- JDK(>=1.7)

We need JDK to compile the Java source files in the tests directory. 

# Build

```shell
$ git clone https://github.com/hanakeichen/rsvm.git
$ cd rsvm
$ cargo build && cargo test
```

# Run

Please make sure you have already run **cargo test**.

```shell
$ ./target/debug/rava --class-path ./tests/classes rsvm.HelloRSVM
Hello rsvm.
```

# Usage

    Usage of rava:
      rava [OPTIONS] <MAIN_CLASS>

      Arguments:
        <MAIN_CLASS>  The main class

      Options:
        [-c, --class-path <CLASS_PATH>]        Class search path of directories and jar files
        [-h, --help]                           Print help
        [-V, --version]                        Print version


# Test

```shell
$ cargo test
```

# Limitations

The interpreter implements direct threading in Rust using inline assembly, but it violates the [Rules](https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly). So it is not guaranteed to work.
