extern crate hidapi;
mod co2_monitor;
use co2_monitor::Co2Monitor;

fn main() {
    let vendor_id = 0x04D9;
    let product_id = 0xA052;

    let mut co2_monitor = Co2Monitor::new(vendor_id, product_id);
    co2_monitor.init();
    if let Some((co2, temp)) = co2_monitor.read() {
        println!("co2:{} temp:{:.1}", co2, temp);
    }
}
