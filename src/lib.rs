pub mod errors;
pub mod protocol;
pub mod requests;
pub mod responses;
use std::net::Ipv4Addr;
use std::num::NonZeroU16;
use std::time::Duration;
use std::{fs::read_to_string, io::ErrorKind};

use protocol::MappingProtocol;
use requests::external_address_request::ExternalAddressRequest;
use requests::mapping_request::MappingRequest;
use requests::unmap_all_request::UnmapAllPortsRequest;
use requests::unmap_request::UnmapPortRequest;
use responses::parse_raw_response;
use socket2::Socket;
use tokio::net::UdpSocket;
use tracing::{event, Level};

use crate::errors::NATPMPError;
use crate::requests::Request;
use crate::responses::{MappingResponse, Response};

const VERSION: u8 = 0;
const NATPMP_PORT: u16 = 5351;

const PATH_PROC_NET_ROUTE: &str = "/proc/net/route";

fn get_gateway_addr() -> Result<Ipv4Addr, NATPMPError> {
    let route_text = read_to_string(PATH_PROC_NET_ROUTE).unwrap_or_default();

    // skip title
    for line in route_text.lines().skip(1) {
        // skip name
        let mut iter = line.split('\t').skip(1).map(str::trim);

        let destination = iter.next().map(|v| u32::from_str_radix(v, 16));
        let gateway = iter.next().map(|v| u32::from_str_radix(v, 16));

        if let (Some(Ok(d)), Some(Ok(g))) = (destination, gateway) {
            if d == 0 && g != 0 {
                return Ok(g.to_be().into());
            }
        }
    }

    Err(NATPMPError::Generic("No default gateway found".into()))
}

/// Takes a gateway address string and returns a non-blocking UDP
/// socket to communicate with its NAT-PMP implementation on `crate::NATPMP_PORT`.
///
/// # Example:
/// ```
/// let socket = connect_gateway("10.0.1.1")?;
/// ```
// async fn connect_gateway(gateway_ip: Ipv4Addr) -> Result<UdpSocket, NATPMPError> {
//     // old code because I'll need it later

//     // bind to the multicast address
//     // Source: https://www.rfc-editor.org/rfc/rfc6886#page-6:~:text=Clients%20should%20therefore%0A%20%20%20bind%20specifically%20to%20224.0.0.1%3A5350
//     // const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);
//     // ^ Above doesn't work. TODO...

//     // use socket so that we don't have to bind to any address explicitely
//     let socket = Socket::new(
//         socket2::Domain::IPV4,
//         socket2::Type::DGRAM,
//         Some(socket2::Protocol::UDP),
//     )?;

//     socket.set_nonblocking(true)?;

//     let socket = UdpSocket::from_std(std::net::UdpSocket::from(socket))?;

//     socket.connect((gateway_ip, NATPMP_PORT));

//     Ok(socket)
// }

/// Creates a non-blocking UDP socket. We don't bind to anything as we want to use it to send stuff randomly.
fn build_socket() -> Result<UdpSocket, NATPMPError> {
    // bind to the multicast address
    // Source: https://www.rfc-editor.org/rfc/rfc6886#page-6:~:text=Clients%20should%20therefore%0A%20%20%20bind%20specifically%20to%20224.0.0.1%3A5350
    // const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);
    // ^ Above doesn't work. TODO...

    // use socket so that we don't have to bind to any address explicitely
    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_nonblocking(true)?;

    // can't create a tokio socket from socket2
    let socket = std::net::UdpSocket::from(socket);

    let socket = tokio::net::UdpSocket::from_std(socket)?;

    Ok(socket)
}

/// A high-level function that returns the public interface IP of
/// the current host by querying the NAT-PMP gateway.
///
/// # Arguments
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///              Defaults to 9 as per specification.
///
/// # Returns
/// The IP address of the gateway
///
/// # Errors
/// Described by the Error component of the Result
pub async fn get_public_address(
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<Ipv4Addr, NATPMPError> {
    let gateway_ip = match gateway_ip {
        Some(g) => g,
        None => get_gateway_addr()?,
    };

    let address_response = send_request_with_retry(
        gateway_ip,
        ExternalAddressRequest::new(),
        retry.unwrap_or(9),
    )
    .await;

    address_response.map(|r| r.ipv4_address)
}

/// A high-level wrapper to `map_port()` that requests a mapping
/// for a public TCP port on the NAT to a private TCP port on this host.
///
/// Returns the complete response on success.
///
/// # Arguments
/// * `public_port` - the public port of the mapping requested
/// * `private_port` - the private port of the mapping requested
/// * `lifetime` - the duration of the mapping in seconds.
///                 Defaults to 7200, per specification.
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///             Defaults to 9 as per specification.
///
/// # Errors
///
/// Described by the Error component of the Result
pub async fn map_tcp_port(
    public_port: Option<NonZeroU16>,
    private_port: NonZeroU16,
    lifetime: Option<u32>,
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<MappingResponse, NATPMPError> {
    map_port(
        MappingProtocol::TCP,
        private_port,
        public_port,
        lifetime,
        gateway_ip,
        retry,
    )
    .await
}

/// A high-level wrapper to `map_port()` that requests a mapping
/// for a public UDP port on the NAT to a private UDP port on this host.
///
/// Returns the complete response on success.
///
/// # Arguments
/// * `public_port` - the public port of the mapping requested
/// * `private_port` - the private port of the mapping requested
/// * `lifetime` - the duration of the mapping in seconds.
///                 Defaults to 7200, per specification.
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///             Defaults to 9 as per specification.
///
/// # Errors
///
/// Described by the Error component of the Result
pub async fn map_udp_port(
    public_port: Option<NonZeroU16>,
    private_port: NonZeroU16,
    lifetime: Option<u32>,
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<MappingResponse, NATPMPError> {
    map_port(
        MappingProtocol::UDP,
        private_port,
        public_port,
        lifetime,
        gateway_ip,
        retry,
    )
    .await
}

/// A function to map `public_port` to `private_port` of protocol.
///
/// Returns the complete response on success.
///
/// # Arguments
/// * `protocol` - `Protocol::TCP` or `Protocol::UDP`
/// * `private_port` - the private port of the mapping requested
/// * `public_port` - the public port of the mapping requested
/// * `lifetime` - the duration of the mapping in seconds.
///                 Defaults to 7200, per specification.
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///             Defaults to 9 as per specification.
///
/// # Errors
///
/// Described by the Error component of the Result
#[allow(clippy::let_and_return)]
pub async fn map_port(
    protocol: MappingProtocol,
    private_port: NonZeroU16,
    public_port: Option<NonZeroU16>,
    lifetime: Option<u32>,
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<MappingResponse, NATPMPError> {
    let gateway_ip = match gateway_ip {
        Some(g) => g,
        None => get_gateway_addr()?,
    };

    let port_mapping_request = MappingRequest::new(
        protocol,
        private_port,
        public_port.map_or(0, Into::into),
        lifetime.unwrap_or(7200),
    );

    let port_mapping_response =
        send_request_with_retry(gateway_ip, port_mapping_request, retry.unwrap_or(9)).await;

    port_mapping_response
}

/// A function to unmap a `private_port` of a protocol.
///
/// Returns the complete response on success.
///
/// # Arguments
/// * `protocol` - `Protocol::TCP` or `Protocol::UDP`
/// * `private_port` - the private port of the mapping requested
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///             Defaults to 9 as per specification.
///
/// # Errors
///
/// Described by the Error component of the Result
#[allow(clippy::let_and_return)]
pub async fn unmap_port(
    protocol: MappingProtocol,
    private_port: NonZeroU16,
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<MappingResponse, NATPMPError> {
    let gateway_ip = match gateway_ip {
        Some(g) => g,
        None => get_gateway_addr()?,
    };

    let port_mapping_request = UnmapPortRequest::new(protocol, private_port);

    let port_mapping_response =
        send_request_with_retry(gateway_ip, port_mapping_request, retry.unwrap_or(9)).await;

    port_mapping_response
}

/// A function to unmap all private ports of a protocol.
///
/// Returns the complete response on success.
///
/// # Arguments
/// * `protocol` - `Protocol::TCP` or `Protocol::UDP`
/// * `gateway_ip` - the IP to the NAT-PMP compatible gateway, or autodetect teh IP via `get_gateway_addr()`
/// * `retry` - the number of times to retry the request if unsuccessful.
///             Defaults to 9 as per specification.
///
/// # Errors
///
/// Described by the Error component of the Result
#[allow(clippy::let_and_return)]
pub async fn unmap_all_ports(
    protocol: MappingProtocol,
    gateway_ip: Option<Ipv4Addr>,
    retry: Option<u32>,
) -> Result<MappingResponse, NATPMPError> {
    let gateway_ip = match gateway_ip {
        Some(g) => g,
        None => get_gateway_addr()?,
    };

    let port_mapping_request = UnmapAllPortsRequest::new(protocol);

    let port_mapping_response =
        send_request_with_retry(gateway_ip, port_mapping_request, retry.unwrap_or(9)).await;

    port_mapping_response
}

async fn send_request(
    gateway_socket: &UdpSocket,
    gateway_ip: Ipv4Addr,
    request: &(impl Request + zerocopy::AsBytes),
) -> Result<usize, NATPMPError> {
    gateway_socket
        .send_to(request.as_bytes(), (gateway_ip, NATPMP_PORT))
        .await
        .map_err(Into::into)
}

async fn send_request_with_retry<R: Request + zerocopy::AsBytes>(
    gateway_ip: Ipv4Addr,
    request: R,
    retry: u32,
) -> Result<R::Response, NATPMPError> {
    // start at 250 milliseconds wait and then increase
    // Source: https://www.rfc-editor.org/rfc/rfc6886#page-6:~:text=and%20waits%20250%20ms%20for%20a%20response.%20%20If%20no%0A%20%20%20NAT%2DPMP%20response%20is%20received%20from%20the%20gateway%20after%20250%20ms%2C%20the%0A%20%20%20client%20retransmits%20its%20request%20and%20waits%20500%20ms
    const BASE_TIMEOUT: u64 = 250;

    let socket = build_socket()?;

    // buffer is at minimum the size of a response, e.g. 12 for external address or 16 for port mapping
    // an error is 8, so that'll always fit
    let mut buffer = R::Response::get_buffer();

    for tries in 1..=retry {
        let _size = send_request(&socket, gateway_ip, &request).await?;

        match tokio::time::timeout(
            Duration::from_millis(BASE_TIMEOUT * 2u64.pow(tries)),
            socket.recv_from(&mut buffer),
        )
        .await
        {
            Ok(Ok((_size, from))) => {
                // ignore response if it isn't from the gateway we sent it to
                // source: https://www.rfc-editor.org/rfc/rfc6886#page-6:~:text=Upon%20receiving%20a%20response%20packet%2C%20the%20client%20MUST%20check%20the%20source%20IP%0A%20%20%20address%2C%20and%20silently%20discard%20the%20packet%20if%20the%20address%20is%20not%20the%0A%20%20%20address%20of%20the%20gateway%20to%20which%20the%20request%20was%20sent.
                if from.ip() == gateway_ip {
                    return parse_raw_response::<R>(&request, &buffer.freeze());
                }

                // ignore try. We clear the buffer as recv_from might return less next time and overwrite only the first bytes.
                buffer.clear();
            },
            Ok(Err(error)) => {
                if error.kind() != ErrorKind::WouldBlock {
                    event!(Level::ERROR, ?error);
                    // TODO if error = ICMP Port Unreachable we should break, else continue
                    break;
                }
            },
            Err(_) => {
                event!(Level::WARN, "Connection timed out, try {}/{}", tries, retry);
            },
        }
    }

    Err(NATPMPError::Unsupported)
}
