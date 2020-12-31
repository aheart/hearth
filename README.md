Hearth
------
Hearth is an SSH based real-time linux server monitoring solution.

[![Build Status](https://travis-ci.com/aheart/hearth.svg?branch=master)](https://travis-ci.com/aheart/hearth)
[![Coverage Status](https://coveralls.io/repos/github/aheart/hearth/badge.svg?branch=master)](https://coveralls.io/github/aheart/hearth?branch=master)

It is particularly useful if you:
* are monitoring 2-15 servers
* don't want to (or can't) install monitoring software on each server
* don't care about historical data (>30 minutes)
* want to make the charts available to a number of people via a Web UI

Pre-built binaries are available for [linux](https://github.com/aheart/hearth/releases).

### Features
Examine health and load patterns via a number of metrics across a small cluster.

For demo purposes all the charts on the screenshot below are showing data for the same machine under different hostnames.
![screenshot](./assets/screenshot.png)


##### CPU
![screenshot](./assets/cpu.gif)

##### Memory
![screenshot](./assets/ram.gif)

##### Disk
![screenshot](./assets/disk.gif)

##### Network
![screenshot](./assets/network.gif)

##### Load Average
![screenshot](./assets/load-average.gif)

##### Disk Space
![screenshot](./assets/space.gif)


### Setup
1. [Download](https://github.com/aheart/hearth/releases) and extract Hearth
2. Adjust **config.toml** to your needs.
3. Run the hearth binary and navigate your browser to the ip/port configured in **config.toml**.


### Limitations
* Data can only be retrieved via SSH.
* Only a single network interface can be monitored per server.
* Only a single disk can be monitored per server.
* Current UI only works well on wide screens

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
