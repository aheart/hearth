use ssh2::{Channel, Session};
use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;
use log::{info, error, debug};

pub struct SshClient {
    username: String,
    hostname: String,
    port: usize,
    tcp: Option<TcpStream>,
    session: Option<Session>,
    cpus: usize,
    uptime_seconds: u64,
}

impl SshClient {
    pub fn new(username: String, hostname: String, port: usize) -> Self {
        SshClient {
            username,
            hostname,
            port,
            tcp: None,
            session: None,
            cpus: 0, //@TODO Move to cpu module. Can be extracted from /proc/stat
            uptime_seconds: 0,
        }
    }

    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }

    pub fn get_cpus(&self) -> usize {
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
        let raw_uptime = self.run("cat /proc/uptime")
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
    fn connect(&mut self) {
        info!("[{}] Connecting.", self.hostname);
        let address = format!("{}:{}", self.hostname, self.port);
        let tcp = TcpStream::connect(address.clone());

        match tcp {
            Ok(tcp) => self.tcp = Some(tcp),
            Err(e) => {
                error!(
                    "Failed to create TCP Connection for {}, error: {:?}",
                    address, e
                );
                self.tcp = None;
                self.session = None;
                return;
            }
        };
        debug!("[{}] Initiating session", self.hostname);
        let mut session = Session::new().unwrap();

        {
            let stream = self.tcp.as_ref().unwrap();
            debug!("[{}] Performing handshake", self.hostname);
            let handshake = session.handshake(stream);
            match handshake {
                Ok(_) => {}
                Err(e) => {
                    error!("Handshake failed for {}, error: {:?}", address, e);
                    self.session = None;
                    return;
                }
            };
        }

        debug!("[{}] Authenticating", self.hostname);
        match session.userauth_agent(&*self.username) {
            Ok(_) => {}
            Err(e) => {
                error!("Authentication failed for {}, error: {:?}", address, e);
            }
        };

        if !session.authenticated() {
            error!("Authentication failed for {}", address);
            self.session = None;
            return;
        }

        self.session = Some(session);
        info!("[{}] Connection established", self.hostname);

        let cpus = self.run("nproc").unwrap_or_else(|_| "0".to_string());
        self.cpus = usize::from_str(cpus.trim_end()).unwrap_or(0);

        self.update_uptime();
    }

    /// Get channel to run command
    fn channel(&mut self) -> Result<Channel<'_>, Box<dyn (::std::error::Error)>> {
        match self.session {
            Some(_) => {}
            None => self.connect(),
        };

        let session = self.session.as_ref();
        if session.is_none() {
            return Err(From::from(format!(
                "[{}] Attempt to connect has failed",
                self.hostname
            )));
        }
        let session = session.unwrap();
        Ok(session.channel_session()?)
    }
}
