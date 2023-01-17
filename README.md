## Overview

This is a NSS module for [innernet](https://github.com/tonarino/innernet) host
resolution.  This provides an alternative to innernet editing the /etc/hosts
file.

## Installation

1. Install the library:
    ```bash
    $ cargo build --release
    $ sudo install -m0755 target/release/libnss_innernet.so /usr/lib/libnss_innernet.so.2
    ```
2. Add the innernet module to /etc/nsswitch.conf.  For example:
    ```
    hosts: mymachines innernet resolve [!UNAVAIL=return] files myhostname dns
    ```

## Usage

```bash
$ innernet list -s | grep fire
  | ◉ 172.28.208.2: fire (...)
$ getent hosts fire.example.wg
172.28.208.2    fire.example.wg
$ getent hosts 172.28.208.2
172.28.208.2    fire.example.wg
```

## How

The library simply reads the json files in /var/lib/innernet to determine host
addresses.

It may be desireable in future to provide an option to ask the innernet-server
directly via the HTTP API.  The `/v1/user/status` endpoint provides the same
JSON that can be found locally on the system.
