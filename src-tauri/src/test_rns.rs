use reticulum_rs::transport::{Transport, TransportConfig};
use reticulum_rs::identity::PrivateIdentity;
use reticulum_rs::destination::{Destination, DestinationName};
use reticulum_rs::iface::AutoInterface;

#[tokio::main]
async fn main() {
    let mut transport = Transport::new(TransportConfig::default());
    let auto_iface = AutoInterface::new("test");
    transport.add_interface(auto_iface);
    
    let identity = PrivateIdentity::new_from_name("test");
    let dest_name = DestinationName::new("app", "aspect");
    let mut destination = transport.add_destination(identity, dest_name).await;
    
    // Check if there is a receiver method
    // let mut rx = destination.receiver(); 
}
