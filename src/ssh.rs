use crate::config::AuthMethod;
use log::{debug, info};
use ssh2::{Channel, Session};
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::str::FromStr;

pub struct SshClient {
    username: String,
    auth_method: AuthMethod,
    hostname: String,
    port: usize,
    session: Option<Session>,
    cpus: u8,
    uptime_seconds: u64,
    ip: String,
}

impl SshClient {
    pub fn new(username: String, auth_method: AuthMethod, hostname: String, port: usize) -> Self {
        SshClient {
            username,
            auth_method,
            hostname,
            port,
            session: None,
            cpus: 0, //@TODO Move to cpu module. Can be extracted from /proc/stat
            uptime_seconds: 0,
            ip: "".to_string(),
        }
    }

    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }

    pub fn get_ip(&self) -> &str {
        &self.ip
    }

    pub fn get_cpus(&self) -> u8 {
        self.cpus
    }

    pub fn get_uptime(&self) -> u64 {
        self.uptime_seconds
    }

    /// Run command on server and if it fails invalidate the session
    pub fn run(&mut self, command: &str) -> Result<String, Box<dyn (::std::error::Error)>> {
        self.exec(command).map_err(move |error| {
            self.session = None;
            error
        })
    }

    pub fn update_uptime(&mut self) {
        let raw_uptime = self
            .run("cat /proc/uptime")
            .unwrap_or_else(|_| "0".to_string());
        let (parts, _): (Vec<&str>, Vec<&str>) = raw_uptime.split(' ').partition(|s| !s.is_empty());
        let uptime_seconds = parts.get(0).unwrap_or(&"0"); // and_then?
        let uptime_seconds = f64::from_str(uptime_seconds).unwrap_or(0.0);
        self.uptime_seconds = uptime_seconds as u64;
    }

    fn exec(&mut self, command: &str) -> Result<String, Box<dyn (::std::error::Error)>> {
        let mut channel = self.channel()?;
        channel.exec(command)?;

        let mut result = String::new();
        channel.read_to_string(&mut result)?;
        Ok(result)
    }

    /// Connect to server, authenticate and fetch the number of CPUs
    fn init(&mut self) -> Result<(), Box<dyn (::std::error::Error)>> {
        self.session = None;
        info!("[{}] Connecting.", self.hostname);
        let session = self.try_connect()?;

        self.session = Some(session);
        info!("[{}] Connection established", self.hostname);

        let cpus = self.run("nproc").unwrap_or_else(|_| "0".to_string());
        self.cpus = u8::from_str(cpus.trim_end()).unwrap_or(0);
        self.update_uptime();
        Ok(())
    }

    fn try_connect(&mut self) -> Result<Session, Box<dyn (::std::error::Error)>> {
        let address = format!("{}:{}", self.hostname, self.port);
        let mut socket_address = address.to_socket_addrs()?;
        let socket_address = socket_address
            .next()
            .ok_or_else(|| format!("Please verify that the address {} is valid", address))?;

        debug!("[{}] Opening TCP connection", self.hostname);
        let timeout = ::std::time::Duration::from_secs(1);
        let tcp = TcpStream::connect_timeout(&socket_address, timeout)?;
        self.ip = tcp.peer_addr()?.ip().to_string();

        debug!("[{}] Initializing session", self.hostname);
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.set_timeout(5000);

        debug!("[{}] Performing handshake", self.hostname);
        session.handshake()?;

        debug!("[{}] Authenticating", self.hostname);
        match &self.auth_method {
            AuthMethod::SshAgent => {
                session.userauth_agent(&*self.username)?;
            }
            AuthMethod::PubKey(config) => {
                session.userauth_pubkey_file(
                    &*self.username,
                    config.public_key_path(),
                    config.private_key_path(),
                    config.passphrase(),
                )?;
            }
        }

        Ok(session)
    }

    /// Get channel to run command
    fn channel(&mut self) -> Result<Channel, Box<dyn (::std::error::Error)>> {
        match self.session {
            Some(_) => {}
            None => self.init()?,
        };

        let session = self.session.as_ref();
        if session.is_none() {
            return Err(From::from("Attempt to connect has failed"));
        }
        let session = session.expect("There is a bug in the SSH client");
        Ok(session.channel_session()?)
    }
}
