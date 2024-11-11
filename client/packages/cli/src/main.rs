use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket},
    time::{Duration, SystemTime},
};

use argh::FromArgs;
use color_eyre::eyre::WrapErr;
use polling::{Event, Events, Poller};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use flatbuffers_structs::net_protocol::{ConfigArgs, Endpoint, HandshakeArgs};
use protocol::connection::Connection;
use vita_virtual_device::{VitaDevice, VitaVirtualDevice};

mod config;

/// Create a virtual controller and fetch its data from a Vita
/// over the network.
#[derive(FromArgs)]
struct Args {
    /// port to connect to (default: 5000)
    #[argh(option, short = 'p')]
    port: Option<u16>,

    /// buttons and touchpads config (default: standart)
    #[argh(option, short = 'c')]
    configuration: Option<String>,

    /// polling interval in microseconds (minimum = 6000)
    #[argh(option)]
    polling_interval: Option<u64>,

    /// enable debug mode
    #[argh(switch, short = 'd')]
    debug: bool,

    /// IP address of the Vita to connect to
    #[argh(positional)]
    ip: Option<String>,

    /// show version information
    #[argh(switch, short = 'v')]
    version: bool,

    /// print sample config file
    #[argh(switch, short = 's')]
    sample_config: bool,
}

fn filter_udp_nonblocking_error(
    err: std::io::Error,
) -> Result<(usize, SocketAddr), std::io::Error> {
    if err.kind() == std::io::ErrorKind::WouldBlock {
        Ok((0, SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)))
    } else {
        Err(err)
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let config;

    let mut args: Args = argh::from_env();
    
    // Do not load existing config while printing sample config
    if !args.sample_config {
        // Load existing config if any
        config = config::load_config().wrap_err("Failed to load configuration")?;
    } else {
        // Show sample configuration
        config::print_sample_config();
        return Ok(());
    }

    args.port = args.port.or(config.port);
    args.configuration = args.configuration.or(config.configuration);
    args.polling_interval = args.polling_interval.or(config.polling_interval);
    args.ip = args.ip.or(config.ip);
    args.debug = args.debug || config.debug.unwrap_or(false);

    if args.version {
        let naive_datetime = NaiveDateTime::parse_from_str(env!("BUILD_TIMESTAMP"), "%Y-%m-%dT%H:%M:%S%.fZ")
            .expect("Failed to parse source timestamp");
        let datetime_local: DateTime<Local> = Local.from_utc_datetime(&naive_datetime);
        let human_readable_time = datetime_local.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("\nVitaPad Client version {}", env!("CARGO_PKG_VERSION"));
        println!("Commit {} {}", env!("GIT_BRANCH"), env!("GIT_COMMIT"));
        println!("Built time {}", human_readable_time);
        return Ok(());
    }

    if args.debug {
        std::env::set_var("RUST_LOG", "trace");
    }
    pretty_env_logger::init();

    let ip_str = match &args.ip {
        Some(ip) => ip,
        _none => {
            eprintln!("Error: IP address is required.");
            return Ok(());
        }
    };
    
    const NET_PORT: u16 = 5000;
    const TIMEOUT: Duration = Duration::from_secs(25);
    const BUFFER_SIZE: usize = 2048;
    const MIN_POLLING_RATE: u64 = (1 * 1000 / 250) * 1000;

    let remote_port = args.port.unwrap_or(NET_PORT);
    let polling_interval = args
        .polling_interval
        .map(|v| v.max(MIN_POLLING_RATE))
        .unwrap_or(MIN_POLLING_RATE);

    let addr = SocketAddr::V4(SocketAddrV4::new(
        ip_str.parse().wrap_err("invalid IPv4 address")?,
        remote_port
    ));
    let mut conn = Connection::new();

    let mut ctrl_socket = TcpStream::connect_timeout(&addr, TIMEOUT).wrap_err(
        "Failed to connect to device, please check that the IP address and port are correct",
    )?;

    let pad_socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).wrap_err("Failed to bind UDP socket")?;

    pad_socket
        .set_nonblocking(true)
        .wrap_err("Failed to set non-blocking mode on socket")?;

    let bound_port = pad_socket
        .local_addr()
        .expect("Failed to get connection info for data socket")
        .port();

    conn.send_handshake(HandshakeArgs {
        endpoint: Endpoint::Client,
        port: bound_port,
        ..Default::default()
    });

    ctrl_socket
        .write_all(conn.retrieve_out_data().as_slice())
        .wrap_err("Failed to send handshake to Vita")?;

    log::info!("Handshake sent to Vita");

    log::info!("Waiting for handshake response from Vita");

    let mut buf = [0; BUFFER_SIZE];

    let len = ctrl_socket
        .read(&mut buf)
        .wrap_err("Failed to read handshake response from Vita")?;

    log::info!("Handshake response received from Vita");

    conn.receive_data(&buf[..len]);
    let event = conn
        .events()
        .next()
        .expect("No handshake response received");
    let handshake_response = match event {
        Ok(protocol::events::Event::HandshakeResponseReceived { handshake }) => handshake,
        Err(e) => return Err(e).wrap_err("Failed to receive handshake response from Vita"),
        _ => unimplemented!("Unexpected event received"),
    };
    let heartbeat_freq = handshake_response.heartbeat_freq;
    log::debug!("Heartbeat frequency: {}", heartbeat_freq);

    // We just send it to bypass firewall
    conn.send_heartbeat();
    pad_socket
        .send_to(conn.retrieve_out_data().as_slice(), addr)
        .wrap_err("Failed to send heartbeat to Vita")?;

    log::info!("Opened port for data on {}", bound_port);

    let mut last_time = SystemTime::now();

    let mut device = VitaDevice::create(args.configuration.as_deref().unwrap_or("standart"))
        .wrap_err("Failed to create virtual device, please check uinput permissions")?;

    let identfiers = device.identifiers().map(|ids| ids.join(", ".as_ref()));
    log::info!("Virtual device created");
    if let Some(identifiers) = identfiers {
        println!(
            "Virtual device created with identifiers: {}",
            identifiers.to_string_lossy()
        );
    }

    println!("Connection established, press Ctrl+C to exit");

    if polling_interval < MIN_POLLING_RATE {
        log::warn!(
            "Polling interval is too low, it has been set to {} microseconds",
            MIN_POLLING_RATE
        );
    }

    if polling_interval != MIN_POLLING_RATE {
        conn.send_config(ConfigArgs {
            polling_interval: polling_interval,
            ..Default::default()
        });
        ctrl_socket
            .write_all(conn.retrieve_out_data().as_slice())
            .wrap_err("Failed to send configuration to Vita")?;
    }

    let poller = Poller::new().wrap_err("Failed to create poller")?;
    unsafe {
        poller
            .add_with_mode(&pad_socket, Event::readable(1), polling::PollMode::Level)
    }
    .wrap_err("Failed to add socket to poller")?;

    let mut events = Events::new();
    let mut last_timestamp = 0;
    loop {
        log::trace!("Polling");
        let timeout = Duration::from_secs(
            (heartbeat_freq.saturating_sub(5) as u64)
                .saturating_sub(last_time.elapsed().unwrap().as_secs()),
        );
        poller
            .wait(&mut events, Some(timeout))
            .wrap_err("Failed to poll")?;

        if last_time
            .elapsed()
            .expect("Cannot get elapsed time")
            .as_secs()
            >= (heartbeat_freq.saturating_sub(5)).into()
        {
            log::debug!("Sending heartbeat to Vita");
            conn.send_heartbeat();
            ctrl_socket
                .write_all(conn.retrieve_out_data().as_slice())
                .wrap_err("Failed to send heartbeat to Vita")?;
            log::debug!("Heartbeat sent to Vita");
            last_time = SystemTime::now();
            log::trace!("Last time updated to {last_time:?}");
        }

        if events.len() == 0 {
            continue;
        }

        let (len, _) = pad_socket
            .recv_from(&mut buf)
            .or_else(filter_udp_nonblocking_error)
            .wrap_err("Failed to receive data from Vita")?;
        log::debug!("Received {len} bytes from Vita");

        let received_data = &buf[..len];

        log::trace!("Received bytes from Vita: {received_data:?}");

        if received_data.is_empty() {
            continue;
        }

        conn.receive_data(received_data);

        for event in conn.events() {
            log::debug!("Event received: {event:?}");
            match event {
                Ok(protocol::events::Event::PadDataReceived { data }) => {
                    if data.timestamp <= last_timestamp {
                        log::warn!("Timestamp is not increasing, dropping packet");
                        continue;
                    }
                    last_timestamp = data.timestamp;

                    let report = vita_reports::MainReport::from(data);
                    log::trace!("Sending report to virtual device: {report:?}");
                    device
                        .send_report(report)
                        .wrap_err("Failed to send report to virtual device")?;
                }
                Err(e) => eprintln!("Error when receiving data from Vita: {e}"),
                _ => {}
            }
        }

        events.clear();
    }
}
