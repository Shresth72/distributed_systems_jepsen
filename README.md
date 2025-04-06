<h1 align="">Jepsen Based Distributed Systems Implementation</h1>
<!--
<p align="center">
  <a href="" rel="noopener">
 <img width=200px height=200px src="https://i.imgur.com/6wj0hh6.jpg" alt="Project logo"></a>
</p>
<div align="center">

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![GitHub Issues](https://img.shields.io/github/issues/Shresth72/distributed_systems_jepsen)](https://github.com/Shresth72/distributed_systems_jepsen)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/Shresth72/distributed_systems_jepsen)](https://github.com/kylelobo/The-Documentation-Compendium/pulls)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)

</div>
-->

<p align="">
This is a Node implementation for a Distributed System based on <a href="https://jepsen.io/">Jepsen</a>, along with testing for each part.
    <br> 
</p>

<!--
## 📝 Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [Deployment](#deployment)
- [Usage](#usage)
- [Built Using](#built_using)
- [TODO](../TODO.md)
- [Contributing](../CONTRIBUTING.md)
- [Authors](#authors)
- [Acknowledgments](#acknowledgement)

-->

### About <a name = "about"></a>

An implementation of challenges on the [fly.io](https://fly.io/dist-sys/1/), which is built on Jepsen. Nodes for Distributed Systems are built in Rust binaries, that accepts failures and verification requests from the Maelstrom tester.

<!-- ### 🏁 Getting Started <a name = "getting_started"></a> -->
<!---->
<!-- These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See [deployment](#deployment) for notes on how to deploy the project on a live system. -->

### Prerequisites

Currently, the testing is entirely handeled by the Maelstrom platform written in Clojure, so install the prerequisites required for using it. (Working on building the testing platform for the same)

##### Install OpenJDK

#### Ubuntu: 20.04, 22.04

```bash
sudo apt-get install openjdk-17-jdk
```

#### Arch

```bash
sudo pacman -sS java | grep jre
```

---

##### Or Install with Brew along with Graphviz and Gnuplot

```bash
brew install openjdk graphviz gnuplot
```

##### Download and unpack the Maelstrom tarball file

```bash
wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2

tar -xvjf maelstrom.tar.bz2
```

## 🔧 Running the tests <a name = "tests"></a>

There are 6 challenges that can be tested with the Maelstrom tester.

### Echo Test

```
~/maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10
```

### Unique ID Generation Test

```
~/maelstrom/maelstrom test -w unique-ids --bin target/debug/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
```

### Node Broadcast Test

Increase the node count for Multi Node Broadcast testing.

```
~/maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
```

### Grow Only Counter Test

The Grow Counter is available globally using either the Maelstrom Api or the Udp Server implemented in a binary file. Maelstrom Api one is available in Go, so use the implementation.

```
cargo run --bin gcounter_server

~/maelstrom/maelstrom test -w g-counter --bin target/debug/grow_counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition
```

### Node Logs Test

The Logger is also available with the Maelstrom Api in Go. Increase the Node count for Multi Node Log Testing.

```
cargo run --bin log_server

~/maelstrom/maelstrom test -w kafka --bin target/debug/logs_global --node-count 1 --concurrency 2n --time-limit 20 --rate 1000
```

<!-- ## 🎈 Importance<a name="usage"></a> -->
<!---->
<!-- adding.. -->

## ⛏️ Built Using <a name = "built_using"></a>

- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/) for the UDP Key Value Store Server
- [Maelstrom](https://github.com/jepsen-io/maelstrom) for Testing

## Acknowledgements <a name = "acknowledgement"></a>

Go follow him now, the best rust guy

- John Gjenset - [Johnhoo](https://www.youtube.com/@jonhoo)
