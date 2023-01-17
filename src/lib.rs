extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate libnss;

use std::fs::File;
use std::io::BufReader;

use libnss::host::{AddressFamily, Addresses, Host, HostHooks};
use libnss::interop::Response;

use std::net::IpAddr;

type Error = std::io::Error;
type Result<T> = std::result::Result<T, Error>;

struct InnernetHost;
libnss_host_hooks!(innernet, InnernetHost);

impl HostHooks for InnernetHost {
    fn get_all_entries() -> Response<Vec<Host>> {
        match get_networks() {
            Ok(networks) => {
                let mut hosts = vec![];
                for net in networks {
                    net.collect_hosts(&mut hosts);
                }
                Response::Success(hosts)
            }
            Err(_err) => Response::Unavail,
        }
    }

    fn get_host_by_addr(addr: IpAddr) -> Response<Host> {
        match get_networks() {
            Ok(networks) => {
                for net in networks {
                    match net.host_by_addr(addr) {
                        Ok(Some(host)) => return Response::Success(host),
                        Ok(None) => {}
                        Err(_err) => return Response::Unavail,
                    }
                }
                Response::NotFound
            }
            Err(_err) => Response::Unavail,
        }
    }

    fn get_host_by_name(name: &str, family: AddressFamily) -> Response<Host> {
        let mut split = name.split('.');
        let host = match split.next() {
            Some(v) => v,
            None => return Response::Unavail,
        };
        let network = match split.next() {
            Some(v) => v,
            None => return Response::Unavail,
        };
        if (Some("wg") != split.next()) || (None != split.next()) {
            return Response::Unavail;
        } else if family != AddressFamily::IPv4 {
            return Response::NotFound;
        }

        let net = Network {
            name: network.into(),
        };
        match net.host_by_name(host) {
            Ok(Some(host)) => Response::Success(host),
            Ok(None) => Response::NotFound,
            Err(_err) => Response::Unavail,
        }
    }
}

struct Network {
    name: String,
}

impl Network {
    fn read_config_from_file(self: &Self) -> Result<serde_json::Value> {
        let file = File::open(format!("/var/lib/innernet/{}.json", self.name))?;
        Ok(serde_json::from_reader(BufReader::new(file))?)
    }

    fn host_from_peer(self: &Self, peer: &serde_json::Value) -> Option<Host> {
        if let Some(ipstr) = peer["ip"].as_str() {
            if let Ok(ip) = ipstr.parse() {
                return Some(Host {
                    name: format!("{}.{}.wg", peer["name"].as_str().unwrap_or(""), self.name),
                    addresses: Addresses::V4(vec![ip]),
                    aliases: vec![],
                });
            }
        }
        None
    }

    fn collect_hosts(self: &Self, out: &mut Vec<Host>) {
        if let Ok(info) = self.read_config_from_file() {
            if let Some(peers) = info["peers"].as_array() {
                for peer in peers {
                    if let Some(host) = self.host_from_peer(peer) {
                        out.push(host)
                    }
                }
            }
        }
    }

    fn host_by_name(self: &Self, name: &str) -> Result<Option<Host>> {
        let info = self.read_config_from_file()?;
        if let Some(peers) = info["peers"].as_array() {
            for peer in peers {
                if peer["name"].as_str() == Some(name) {
                    return Ok(self.host_from_peer(peer));
                }
            }
        }
        Ok(None)
    }

    fn host_by_addr(self: &Self, addr: IpAddr) -> Result<Option<Host>> {
        let info = self.read_config_from_file()?;
        if let Some(peers) = info["peers"].as_array() {
            for peer in peers {
                let peer_addr = peer["ip"].as_str().unwrap_or("").parse();
                if peer_addr == Ok(addr) {
                    return Ok(self.host_from_peer(peer));
                }
            }
        }
        Ok(None)
    }
}

fn get_networks() -> Result<Vec<Network>> {
    let mut networks = vec![];
    for entry in std::fs::read_dir("/etc/innernet")? {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                if let Some(fname) = entry.file_name().to_str() {
                    networks.push(Network {
                        name: fname.trim_end_matches(".conf").into(),
                    });
                }
            }
        }
    }
    Ok(networks)
}
