use crate::fmt;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6};
use crate::os::uefi;
use crate::os::uefi::raw::protocols::{tcp4, tcp6};
use crate::sys::unsupported;
use crate::time::Duration;

pub struct TcpStream {
    inner: uefi_tcp4::Tcp4Protocol,
}

impl TcpStream {
    fn new(inner: uefi_tcp4::Tcp4Protocol) -> Self {
        Self { inner }
    }

    pub fn connect(_: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        todo!()
    }

    pub fn connect_timeout(_: &SocketAddr, _: Duration) -> io::Result<TcpStream> {
        todo!()
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        unimplemented!()
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        unimplemented!()
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        unimplemented!()
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        unimplemented!()
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }

    pub fn read(&self, _: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }

    pub fn read_vectored(&self, _: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unimplemented!()
    }

    pub fn is_read_vectored(&self) -> bool {
        unimplemented!()
    }

    pub fn write(&self, _: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    pub fn write_vectored(&self, _: &[IoSlice<'_>]) -> io::Result<usize> {
        unimplemented!()
    }

    pub fn is_write_vectored(&self) -> bool {
        unimplemented!()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        Ok(SocketAddr::from(self.inner.remote_socket()?))
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        Ok(SocketAddr::from(self.inner.station_socket()?))
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match how {
            Shutdown::Read => unsupported(),
            Shutdown::Write => unsupported(),
            Shutdown::Both => self.inner.close(false),
        }
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        unimplemented!()
    }

    // Seems to be similar to abort_on_close option in `EFI_TCP6_PROTOCOL->Close()`
    pub fn set_linger(&self, _: Option<Duration>) -> io::Result<()> {
        todo!()
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        todo!()
    }

    // Seems to be similar to `EFI_TCP6_OPTION->EnableNagle`
    pub fn set_nodelay(&self, _: bool) -> io::Result<()> {
        todo!()
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        todo!()
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        unimplemented!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unimplemented!()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        unimplemented!()
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        todo!()
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub struct TcpListener {
    inner: uefi_tcp4::Tcp4Protocol,
}

impl TcpListener {
    fn new(inner: uefi_tcp4::Tcp4Protocol) -> Self {
        Self { inner }
    }

    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        let addr = addr?;
        match addr {
            SocketAddr::V4(x) => {
                let handles = uefi::env::locate_handles(tcp4::SERVICE_BINDING_PROTOCOL_GUID)?;

                // Try all handles
                for handle in handles {
                    let service_binding = uefi_service_binding::ServiceBinding::new(
                        tcp4::SERVICE_BINDING_PROTOCOL_GUID,
                        handle,
                    );
                    let tcp4_protocol = match uefi_tcp4::Tcp4Protocol::create(service_binding) {
                        Ok(x) => x,
                        Err(e) => {
                            println!("Error creating Protocol from Service Binding: {:?}", e);
                            continue;
                        }
                    };

                    // Not sure about Station/Remote address yet
                    match tcp4_protocol.config(
                        true,
                        false,
                        x,
                        &Ipv4Addr::new(255, 255, 255, 0),
                        &SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0),
                    ) {
                        Ok(()) => return Ok(TcpListener::new(tcp4_protocol)),
                        Err(e) => {
                            println!("Error during Protocol Config: {:?}", e);
                            continue;
                        }
                    }
                }

                Err(io::Error::new(io::ErrorKind::Other, "Failed to open any EFI_TCP6_PROTOCOL"))
            }
            SocketAddr::V6(x) => {
                todo!();
                // let handles = uefi::env::locate_handles(tcp6::SERVICE_BINDING_PROTOCOL_GUID)?;

                // // Try all handles
                // for handle in handles {
                //     let service_binding = uefi_service_binding::ServiceBinding::new(
                //         tcp6::SERVICE_BINDING_PROTOCOL_GUID,
                //         handle,
                //     );
                //     let tcp6_protocol = match uefi_tcp6::Tcp6Protocol::create(service_binding) {
                //         Ok(x) => x,
                //         Err(e) => {
                //             println!("Error creating Protocol from Service Binding: {:?}", e);
                //             continue;
                //         }
                //     };

                //     // Not sure about Station/Remote address yet
                //     match tcp6_protocol.config(
                //         false,
                //         &SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, x.port(), 0, 0),
                //         &SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, x.port(), 0, 0),
                //     ) {
                //         Ok(()) => return Ok(TcpListener::new(tcp6_protocol)),
                //         Err(e) => {
                //             println!("Error during Protocol Config: {:?}", e);
                //             continue;
                //         }
                //     }
                // }

                // Err(io::Error::new(io::ErrorKind::Other, "Failed to open any EFI_TCP6_PROTOCOL"))
            }
        }
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        Ok(SocketAddr::from(self.inner.station_socket()?))
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let new_protocol = self.inner.accept()?;
        let socket_addr = new_protocol.remote_socket()?;
        let stream = TcpStream::new(new_protocol);
        Ok((stream, SocketAddr::from(socket_addr)))
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        unimplemented!()
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        unimplemented!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        unimplemented!()
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        unimplemented!()
    }

    // Should be best to just implment TCPv6 for now
    pub fn only_v6(&self) -> io::Result<bool> {
        Ok(true)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        unimplemented!()
    }

    // Internally TCP6 Protocol is nonblocking
    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        todo!()
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub struct UdpSocket(!);

impl UdpSocket {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        unsupported()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0
    }

    pub fn recv_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0
    }

    pub fn peek_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0
    }

    pub fn send_to(&self, _: &[u8], _: &SocketAddr) -> io::Result<usize> {
        self.0
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        self.0
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        self.0
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        self.0
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0
    }

    pub fn set_broadcast(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        self.0
    }

    pub fn set_multicast_loop_v4(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.0
    }

    pub fn set_multicast_ttl_v4(&self, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.0
    }

    pub fn set_multicast_loop_v6(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.0
    }

    pub fn join_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        self.0
    }

    pub fn join_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn leave_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        self.0
    }

    pub fn leave_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn recv(&self, _: &mut [u8]) -> io::Result<usize> {
        self.0
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        self.0
    }

    pub fn send(&self, _: &[u8]) -> io::Result<usize> {
        self.0
    }

    pub fn connect(&self, _: io::Result<&SocketAddr>) -> io::Result<()> {
        self.0
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

pub struct LookupHost(!);

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.0
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.0
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: &str) -> io::Result<LookupHost> {
        unsupported()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: (&'a str, u16)) -> io::Result<LookupHost> {
        unsupported()
    }
}

#[allow(nonstandard_style)]
pub mod netc {
    pub const AF_INET: u8 = 0;
    pub const AF_INET6: u8 = 1;
    pub type sa_family_t = u8;

    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_addr: in6_addr,
        pub sin6_flowinfo: u32,
        pub sin6_scope_id: u32,
    }

    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr {}

    pub type socklen_t = usize;
}

mod uefi_tcp4 {
    use super::uefi_service_binding::{self, ServiceBinding};
    use crate::io;
    use crate::mem::MaybeUninit;
    use crate::net::{Ipv4Addr, SocketAddrV4};
    use crate::os::uefi;
    use crate::os::uefi::raw::protocols::{
        ip4, managed_network, service_binding, simple_network, tcp4,
    };
    use crate::os::uefi::raw::Status;
    use crate::ptr::NonNull;
    use crate::sys_common::AsInner;

    // FIXME: Discuss what the values these constants should have
    const TYPE_OF_SERVICE: u8 = 8;
    const TIME_TO_LIVE: u8 = 255;

    pub struct Tcp4Protocol {
        protocol: NonNull<tcp4::Protocol>,
        service_binding: ServiceBinding,
        child_handle: NonNull<crate::ffi::c_void>,
    }

    impl Tcp4Protocol {
        fn new(
            protocol: NonNull<tcp4::Protocol>,
            service_binding: ServiceBinding,
            child_handle: NonNull<crate::ffi::c_void>,
        ) -> Self {
            Self { protocol, service_binding, child_handle }
        }

        fn with_child_handle(
            service_binding: ServiceBinding,
            child_handle: NonNull<crate::ffi::c_void>,
        ) -> io::Result<Self> {
            let tcp4_protocol = uefi::env::open_protocol(child_handle, tcp4::PROTOCOL_GUID)?;
            Ok(Self::new(tcp4_protocol, service_binding, child_handle))
        }

        fn get_config_data(&self) -> io::Result<tcp4::ConfigData> {
            let protocol = self.protocol.as_ptr();

            let mut state: MaybeUninit<tcp4::ConnectionState> = MaybeUninit::uninit();
            let mut config_data: MaybeUninit<tcp4::ConfigData> = MaybeUninit::uninit();
            let mut ip4_mode_data: MaybeUninit<ip4::ModeData> = MaybeUninit::uninit();
            let mut mnp_mode_data: MaybeUninit<managed_network::ConfigData> = MaybeUninit::uninit();
            let mut snp_mode_data: MaybeUninit<simple_network::Mode> = MaybeUninit::uninit();

            let r = unsafe {
                ((*protocol).get_mode_data)(
                    protocol,
                    state.as_mut_ptr(),
                    config_data.as_mut_ptr(),
                    ip4_mode_data.as_mut_ptr(),
                    mnp_mode_data.as_mut_ptr(),
                    snp_mode_data.as_mut_ptr(),
                )
            };

            if r.is_error() {
                match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No configuration data is available because this instance hasn’t been started",
                    )),
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "This is NULL"))
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                Ok(unsafe { config_data.assume_init() })
            }
        }

        pub fn create(service_binding: ServiceBinding) -> io::Result<Tcp4Protocol> {
            let child_handle = service_binding.create_child()?;
            Self::with_child_handle(service_binding, child_handle)
        }

        pub fn config(
            &self,
            use_default_address: bool,
            active_flag: bool,
            station_addr: &crate::net::SocketAddrV4,
            subnet_mask: &crate::net::Ipv4Addr,
            remote_addr: &crate::net::SocketAddrV4,
        ) -> io::Result<()> {
            let protocol = self.protocol.as_ptr();

            let mut config_data = tcp4::ConfigData {
                // FIXME: Check in mailing list what traffic_class should be used
                type_of_service: TYPE_OF_SERVICE,
                // FIXME: Check in mailing list what hop_limit should be used
                time_to_live: TIME_TO_LIVE,
                access_point: tcp4::AccessPoint {
                    use_default_address: uefi::raw::Boolean::from(use_default_address),
                    station_address: uefi::raw::Ipv4Address::from(station_addr.ip()),
                    station_port: station_addr.port(),
                    subnet_mask: uefi::raw::Ipv4Address::from(subnet_mask),
                    remote_address: uefi::raw::Ipv4Address::from(remote_addr.ip()),
                    remote_port: remote_addr.port(),
                    active_flag: uefi::raw::Boolean::from(active_flag),
                },
                // FIXME: Maybe provide a rust default one at some point
                control_option: crate::ptr::null_mut(),
            };

            let r = unsafe { ((*protocol).configure)(protocol, &mut config_data) };

            if r.is_error() {
                let e = match r {
                    Status::NO_MAPPING => io::Error::new(
                        io::ErrorKind::Other,
                        "The underlying IPv6 driver was responsible for choosing a source address for this instance, but no source address was available for use",
                    ),
                    Status::INVALID_PARAMETER => {
                        io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER")
                    }
                    Status::ACCESS_DENIED => io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Configuring TCP instance when it is configured without calling Configure() with NULL to reset it",
                    ),
                    Status::UNSUPPORTED => io::Error::new(
                        io::ErrorKind::Unsupported,
                        "One or more of the control options are not supported in the implementation.",
                    ),
                    Status::OUT_OF_RESOURCES => io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough system resources when executing Configure()",
                    ),
                    Status::DEVICE_ERROR => io::Error::new(
                        io::ErrorKind::Other,
                        "An unexpected network or system error occurred",
                    ),
                    _ => io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Unknown Error: {}", r.as_usize()),
                    ),
                };
                Err(e)
            } else {
                Ok(())
            }
        }

        pub fn accept(&self) -> io::Result<Tcp4Protocol> {
            let protocol = self.protocol.as_ptr();

            let accept_event = uefi::thread::Event::create(
                uefi::raw::EVT_NOTIFY_WAIT,
                uefi::raw::TPL_CALLBACK,
                Some(nop_notify4),
                None,
            )?;
            let completion_token = tcp4::CompletionToken {
                event: accept_event.as_raw_event(),
                status: Status::ABORTED,
            };

            let mut listen_token = tcp4::ListenToken {
                completion_token,
                new_child_handle: unsafe {
                    MaybeUninit::<uefi::raw::Handle>::uninit().assume_init()
                },
            };

            let r = unsafe { ((*protocol).accept)(protocol, &mut listen_token) };

            if r.is_error() {
                return match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "This EFI TCPv6 Protocol instance has not been configured",
                    )),
                    Status::ACCESS_DENIED => {
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "EFI_ACCESS_DENIED"))
                    }
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER"))
                    }
                    Status::OUT_OF_RESOURCES => Err(io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough resource to finish the operation",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                };
            }
            println!("Wait");
            accept_event.wait()?;
            println!("Wait Done");

            let r = listen_token.completion_token.status;
            if r.is_error() {
                match r {
                    Status::CONNECTION_RESET => Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        "The accept fails because the
connection is reset either by instance itself or communication peer",
                    )),
                    Status::ABORTED => Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "The accept request has been aborted",
                    )),
                    Status::SECURITY_VIOLATION => Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "The accept operation was failed because of IPSec policy check",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                let child_handle = NonNull::new(listen_token.new_child_handle)
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Null Child Handle"))?;
                Self::with_child_handle(self.service_binding, child_handle)
            }
        }

        pub fn connect(&self) -> io::Result<()> {
            todo!()
        }

        pub fn transmit(&self) -> io::Result<()> {
            todo!()
        }

        pub fn receive(&self) -> io::Result<()> {
            todo!()
        }

        pub fn close(&self, abort_on_close: bool) -> io::Result<()> {
            let protocol = self.protocol.as_ptr();

            let close_event = uefi::thread::Event::create(
                uefi::raw::EVT_NOTIFY_SIGNAL,
                uefi::raw::TPL_CALLBACK,
                Some(nop_notify4),
                None,
            )?;
            let completion_token = tcp4::CompletionToken {
                event: close_event.as_raw_event(),
                status: Status::ABORTED,
            };
            let mut close_token = tcp4::CloseToken {
                abort_on_close: uefi::raw::Boolean::from(abort_on_close),
                completion_token,
            };
            let r = unsafe { ((*protocol).close)(protocol, &mut close_token) };

            if r.is_error() {
                return match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "This EFI TCPv6 Protocol instance has not been configured",
                    )),
                    Status::ACCESS_DENIED => {
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "EFI_ACCESS_DENIED"))
                    }
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER"))
                    }
                    Status::OUT_OF_RESOURCES => Err(io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough resource to finish the operation",
                    )),
                    Status::DEVICE_ERROR => {
                        Err(io::Error::new(io::ErrorKind::NetworkDown, "EFI_DEVICE_ERROR"))
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                };
            }

            close_event.wait()?;

            let r = close_token.completion_token.status;
            if r.is_error() {
                match r {
                    Status::ABORTED => Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "The accept request has been aborted",
                    )),
                    Status::SECURITY_VIOLATION => Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "The accept operation was failed because of IPSec policy check",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                Ok(())
            }
        }

        pub fn remote_socket(&self) -> io::Result<SocketAddrV4> {
            let config_data = self.get_config_data()?;
            Ok(SocketAddrV4::new(
                Ipv4Addr::from(config_data.access_point.remote_address),
                config_data.access_point.remote_port,
            ))
        }

        pub fn station_socket(&self) -> io::Result<SocketAddrV4> {
            let config_data = self.get_config_data()?;
            Ok(SocketAddrV4::new(
                Ipv4Addr::from(config_data.access_point.station_address),
                config_data.access_point.station_port,
            ))
        }
    }

    impl Drop for Tcp4Protocol {
        fn drop(&mut self) {
            let _ = self.service_binding.destroy_child(self.child_handle);
        }
    }

    #[no_mangle]
    pub extern "efiapi" fn nop_notify4(_: uefi::raw::Event, _: *mut crate::ffi::c_void) {}
}

mod uefi_tcp6 {
    use super::uefi_service_binding::ServiceBinding;
    use crate::io;
    use crate::mem::MaybeUninit;
    use crate::net::{Ipv6Addr, SocketAddrV6};
    use crate::os::uefi;
    use crate::os::uefi::raw::protocols::{
        ip6, managed_network, service_binding, simple_network, tcp6,
    };
    use crate::os::uefi::raw::Status;
    use crate::ptr::NonNull;
    use crate::sys_common::AsInner;

    // FIXME: Discuss what the values these constants should have
    const TRAFFIC_CLASS: u8 = 0;
    const HOP_LIMIT: u8 = 255;

    pub struct Tcp6Protocol {
        protocol: NonNull<tcp6::Protocol>,
        service_binding: ServiceBinding,
        child_handle: NonNull<crate::ffi::c_void>,
    }

    impl Tcp6Protocol {
        fn new(
            protocol: NonNull<tcp6::Protocol>,
            service_binding: ServiceBinding,
            child_handle: NonNull<crate::ffi::c_void>,
        ) -> Self {
            Self { protocol, service_binding, child_handle }
        }

        fn with_child_handle(
            service_binding: ServiceBinding,
            child_handle: NonNull<crate::ffi::c_void>,
        ) -> io::Result<Self> {
            let tcp6_protocol = uefi::env::open_protocol(child_handle, tcp6::PROTOCOL_GUID)?;
            Ok(Self::new(tcp6_protocol, service_binding, child_handle))
        }

        fn get_config_data(&self) -> io::Result<tcp6::ConfigData> {
            let protocol = self.protocol.as_ptr();

            let mut state: MaybeUninit<tcp6::ConnectionState> = MaybeUninit::uninit();
            let mut config_data: MaybeUninit<tcp6::ConfigData> = MaybeUninit::uninit();
            let mut ip6_mode_data: MaybeUninit<ip6::ModeData> = MaybeUninit::uninit();
            let mut mnp_mode_data: MaybeUninit<managed_network::ConfigData> = MaybeUninit::uninit();
            let mut snp_mode_data: MaybeUninit<simple_network::Mode> = MaybeUninit::uninit();

            let r = unsafe {
                ((*protocol).get_mode_data)(
                    protocol,
                    state.as_mut_ptr(),
                    config_data.as_mut_ptr(),
                    ip6_mode_data.as_mut_ptr(),
                    mnp_mode_data.as_mut_ptr(),
                    snp_mode_data.as_mut_ptr(),
                )
            };

            if r.is_error() {
                match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No configuration data is available because this instance hasn’t been started",
                    )),
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "This is NULL"))
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                Ok(unsafe { config_data.assume_init() })
            }
        }

        pub fn create(service_binding: ServiceBinding) -> io::Result<Tcp6Protocol> {
            let child_handle = service_binding.create_child()?;
            Self::with_child_handle(service_binding, child_handle)
        }

        pub fn config(
            &self,
            active_flag: bool,
            station_addr: &crate::net::SocketAddrV6,
            remote_addr: &crate::net::SocketAddrV6,
        ) -> io::Result<()> {
            let protocol = self.protocol.as_ptr();

            let mut config_data = tcp6::ConfigData {
                // FIXME: Check in mailing list what traffic_class should be used
                traffic_class: TRAFFIC_CLASS,
                // FIXME: Check in mailing list what hop_limit should be used
                hop_limit: HOP_LIMIT,
                access_point: tcp6::AccessPoint {
                    station_address: uefi::raw::Ipv6Address::from(station_addr.ip()),
                    station_port: station_addr.port(),
                    remote_address: uefi::raw::Ipv6Address::from(remote_addr.ip()),
                    remote_port: remote_addr.port(),
                    active_flag: uefi::raw::Boolean::from(active_flag),
                },
                // FIXME: Maybe provide a rust default one at some point
                control_option: crate::ptr::null_mut(),
            };

            let r = unsafe { ((*protocol).configure)(protocol, &mut config_data) };

            if r.is_error() {
                let e = match r {
                    Status::NO_MAPPING => io::Error::new(
                        io::ErrorKind::Other,
                        "The underlying IPv6 driver was responsible for choosing a source address for this instance, but no source address was available for use",
                    ),
                    Status::INVALID_PARAMETER => {
                        io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER")
                    }
                    Status::ACCESS_DENIED => io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Configuring TCP instance when it is configured without calling Configure() with NULL to reset it",
                    ),
                    Status::UNSUPPORTED => io::Error::new(
                        io::ErrorKind::Unsupported,
                        "One or more of the control options are not supported in the implementation.",
                    ),
                    Status::OUT_OF_RESOURCES => io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough system resources when executing Configure()",
                    ),
                    Status::DEVICE_ERROR => io::Error::new(
                        io::ErrorKind::Other,
                        "An unexpected network or system error occurred",
                    ),
                    _ => io::Error::new(
                        io::ErrorKind::Other,
                        format!("Unknown Error: {}", r.as_usize()),
                    ),
                };
                Err(e)
            } else {
                Ok(())
            }
        }

        pub fn accept(&self) -> io::Result<Tcp6Protocol> {
            let protocol = self.protocol.as_ptr();

            let accept_event = uefi::thread::Event::create(
                uefi::raw::EVT_NOTIFY_SIGNAL,
                uefi::raw::TPL_CALLBACK,
                Some(nop_notify),
                None,
            )?;
            let completion_token = tcp6::CompletionToken {
                event: accept_event.as_raw_event(),
                status: Status::ABORTED,
            };

            let mut listen_token = tcp6::ListenToken {
                completion_token,
                new_child_handle: unsafe {
                    MaybeUninit::<uefi::raw::Handle>::uninit().assume_init()
                },
            };

            let r = unsafe { ((*protocol).accept)(protocol, &mut listen_token) };

            if r.is_error() {
                return match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "This EFI TCPv6 Protocol instance has not been configured",
                    )),
                    Status::ACCESS_DENIED => {
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "EFI_ACCESS_DENIED"))
                    }
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER"))
                    }
                    Status::OUT_OF_RESOURCES => Err(io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough resource to finish the operation",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                };
            }
            println!("Wait");
            // accept_event.wait()?;
            // Seems like a bad idea
            while listen_token.completion_token.status == Status::ABORTED {}
            println!("Wait Done");

            let r = listen_token.completion_token.status;
            if r.is_error() {
                match r {
                    Status::CONNECTION_RESET => Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        "The accept fails because the
connection is reset either by instance itself or communication peer",
                    )),
                    Status::ABORTED => Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "The accept request has been aborted",
                    )),
                    Status::SECURITY_VIOLATION => Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "The accept operation was failed because of IPSec policy check",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                let child_handle = NonNull::new(listen_token.new_child_handle)
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Null Child Handle"))?;
                Self::with_child_handle(self.service_binding, child_handle)
            }
        }

        pub fn connect(&self) -> io::Result<()> {
            todo!()
        }

        pub fn transmit(&self) -> io::Result<()> {
            todo!()
        }

        pub fn receive(&self) -> io::Result<()> {
            todo!()
        }

        pub fn close(&self, abort_on_close: bool) -> io::Result<()> {
            let protocol = self.protocol.as_ptr();

            let close_event = uefi::thread::Event::create(
                uefi::raw::EVT_NOTIFY_SIGNAL,
                uefi::raw::TPL_CALLBACK,
                Some(nop_notify),
                None,
            )?;
            let completion_token = tcp6::CompletionToken {
                event: close_event.as_raw_event(),
                status: Status::ABORTED,
            };
            let mut close_token = tcp6::CloseToken {
                abort_on_close: uefi::raw::Boolean::from(abort_on_close),
                completion_token,
            };
            let r = unsafe { ((*protocol).close)(protocol, &mut close_token) };

            if r.is_error() {
                return match r {
                    Status::NOT_STARTED => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "This EFI TCPv6 Protocol instance has not been configured",
                    )),
                    Status::ACCESS_DENIED => {
                        Err(io::Error::new(io::ErrorKind::PermissionDenied, "EFI_ACCESS_DENIED"))
                    }
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "EFI_INVALID_PARAMETER"))
                    }
                    Status::OUT_OF_RESOURCES => Err(io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "Could not allocate enough resource to finish the operation",
                    )),
                    Status::DEVICE_ERROR => {
                        Err(io::Error::new(io::ErrorKind::NetworkDown, "EFI_DEVICE_ERROR"))
                    }
                    _ => Err(io::Error::new(
                        io::ErrorKind::Uncategorized,
                        format!("Status: {}", r.as_usize()),
                    )),
                };
            }

            close_event.wait()?;

            let r = close_token.completion_token.status;
            if r.is_error() {
                match r {
                    Status::ABORTED => Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "The accept request has been aborted",
                    )),
                    Status::SECURITY_VIOLATION => Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "The accept operation was failed because of IPSec policy check",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Status: {}", r.as_usize()),
                    )),
                }
            } else {
                Ok(())
            }
        }

        pub fn remote_socket(&self) -> io::Result<SocketAddrV6> {
            let config_data = self.get_config_data()?;
            Ok(SocketAddrV6::new(
                Ipv6Addr::from(config_data.access_point.remote_address),
                config_data.access_point.remote_port,
                0,
                0,
            ))
        }

        pub fn station_socket(&self) -> io::Result<SocketAddrV6> {
            let config_data = self.get_config_data()?;
            Ok(SocketAddrV6::new(
                Ipv6Addr::from(config_data.access_point.station_address),
                config_data.access_point.station_port,
                0,
                0,
            ))
        }
    }

    impl Drop for Tcp6Protocol {
        fn drop(&mut self) {
            let _ = self.service_binding.destroy_child(self.child_handle);
        }
    }

    #[no_mangle]
    pub extern "efiapi" fn nop_notify(_: uefi::raw::Event, _: *mut crate::ffi::c_void) {}
}

mod uefi_service_binding {
    use crate::io;
    use crate::mem::MaybeUninit;
    use crate::os::uefi;
    use crate::os::uefi::raw::protocols::service_binding;
    use crate::os::uefi::raw::Status;
    use crate::ptr::NonNull;

    #[derive(Clone, Copy)]
    pub struct ServiceBinding {
        service_binding_guid: uefi::raw::Guid,
        handle: NonNull<crate::ffi::c_void>,
    }

    impl ServiceBinding {
        pub fn new(
            service_binding_guid: uefi::raw::Guid,
            handle: NonNull<crate::ffi::c_void>,
        ) -> Self {
            Self { service_binding_guid, handle }
        }

        pub fn create_child(&self) -> io::Result<NonNull<crate::ffi::c_void>> {
            let service_binding_protocol: NonNull<service_binding::Protocol> =
                uefi::env::open_protocol(self.handle, self.service_binding_guid)?;
            let mut child_handle: MaybeUninit<uefi::raw::Handle> = MaybeUninit::uninit();
            let r = unsafe {
                ((*service_binding_protocol.as_ptr()).create_child)(
                    service_binding_protocol.as_ptr(),
                    child_handle.as_mut_ptr(),
                )
            };

            if r.is_error() {
                match r {
                    Status::INVALID_PARAMETER => {
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "ChildHandle is NULL"))
                    }
                    Status::OUT_OF_RESOURCES => Err(io::Error::new(
                        io::ErrorKind::OutOfMemory,
                        "There are not enough resources available to create the child",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Unknown Error: {}", r.as_usize()),
                    )),
                }
            } else {
                NonNull::new(unsafe { child_handle.assume_init() })
                    .ok_or(io::Error::new(io::ErrorKind::Other, "Null Handle"))
            }
        }

        pub fn destroy_child(&self, child_handle: NonNull<crate::ffi::c_void>) -> io::Result<()> {
            let service_binding_protocol: NonNull<service_binding::Protocol> =
                uefi::env::open_protocol(self.handle, self.service_binding_guid)?;
            let r = unsafe {
                ((*service_binding_protocol.as_ptr()).destroy_child)(
                    service_binding_protocol.as_ptr(),
                    child_handle.as_ptr(),
                )
            };

            if r.is_error() {
                match r {
                    Status::UNSUPPORTED => Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "ChildHandle does not support the protocol that is being removed",
                    )),
                    Status::INVALID_PARAMETER => Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "ChildHandle is not a valid UEFI handle",
                    )),
                    Status::ACCESS_DENIED => Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "The protocol could not be removed from the ChildHandle because its services are being used",
                    )),
                    _ => Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Unknown Error: {}", r.as_usize()),
                    )),
                }
            } else {
                Ok(())
            }
        }
    }
}
