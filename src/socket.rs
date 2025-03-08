use std::os::unix::net::UnixStream;

use crate::{config::Config, Output};

pub fn connect_to_hyprland_socket(config: &Config) -> Option<UnixStream> {
    let address = &config.socket_address;
    log::info!("Connecting to unix socket address {address}...");

    Some(match UnixStream::connect(address) {
        Ok(socket) => socket,
        Err(err) => {
            Output::with_error(&format!("Failed to connect to Unix socket: {err:?}")).print_out();
            return None;
        }
    })
}
