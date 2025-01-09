use std::net::IpAddr;
use tokio_util::codec::Framed;
use tun::{AsyncDevice, Configuration, TunPacketCodec};

pub struct TunInterface {
    pub config: Configuration,
    pub framed: Framed<AsyncDevice, TunPacketCodec>, /*>>,*/
}

impl TunInterface {
    pub fn new(addr: IpAddr) -> TunInterface {
        println!("ip:{}", addr);
        // create TUN device
        let mut config = Configuration::default();
        config
            .address(addr)
            .netmask((255, 255, 255, 0))
            .destination((10, 0, 0, 1))
            .mtu(1200)
            .layer(tun::Layer::L2)
            .up();
        #[cfg(target_os = "linux")]
        config.platform_config(|config| {
            // requiring root privilege to acquire complete functions
            config.ensure_root_privileges(true);
        });
        let dev = tun::create_as_async(&config).unwrap();
        let framed = /*Arc::new(Mutex::new(*/dev.into_framed(); //));
                                                                /*        let (send, recv) = dev.split();*/
        TunInterface { config, framed }
    }
}
