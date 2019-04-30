use ssh2::{Channel, Session};
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::str::FromStr;
use log::{info, error, debug};

pub struct SshClient {
    username: String,
    hostname: String,
    port: usize,
    tcp: Option<TcpStream>,
    session: Option<Session>,
    cpus: u8,
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

    pub fn get_ip(&self) -> Option<String> {
        self.tcp
            .as_ref()
            .and_then(|tcp| tcp.peer_addr().ok())
            .and_then(|socket| Some(socket.ip().to_string()))
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
        self.tcp = None;
        self.session = None;
        info!("[{}] Connecting.", self.hostname);
        let (tcp, session) = match self.try_connect() {
            Ok(t) => t,
            Err(e) => {
                error!(
                    "[{}] Failed to connect to host, error: {:?}",
                    self.hostname, e
                );
                return;
            }
        };

        self.tcp = Some(tcp);
        self.session = Some(session);
        info!("[{}] Connection established", self.hostname);

        let cpus = self.run("nproc").unwrap_or_else(|_| "0".to_string());
        self.cpus = u8::from_str(cpus.trim_end()).unwrap_or(0);
        self.update_uptime();
    }

    fn try_connect(&mut self) -> Result<(TcpStream, Session), Box<dyn (::std::error::Error)>> {
        let address = format!("{}:{}", self.hostname, self.port);
        let mut socket_address = address.to_socket_addrs()?;
        let socket_address = socket_address.next()
            .ok_or_else(|| format!("Please verify that the address {} is valid", address))?;

        debug!("[{}] Opening TCP connection", self.hostname);
        let timeout = ::std::time::Duration::from_secs(1);
        let tcp = TcpStream::connect_timeout(&socket_address, timeout)?;

        debug!("[{}] Initializing session", self.hostname);
        let mut session = Session::new().ok_or_else(|| "Failed to create new session".to_string())?;

        debug!("[{}] Performing handshake", self.hostname);
        session.handshake(&tcp)?;

        debug!("[{}] Authenticating", self.hostname);
        session.userauth_agent(&*self.username)?;

        Ok((tcp, session))
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
                "Attempt to connect has failed",
                self.hostname
            )));
        }
        let session = session.unwrap();
        Ok(session.channel_session()?)
    }
}
