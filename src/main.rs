mod co2_monitor;
use co2_monitor::Co2Monitor;

fn main() {
    let mut co2_monitor = Co2Monitor::new();
    co2_monitor.init();
    if let Ok(data) = co2_monitor.read() {
        println!("co2:{} temp:{:.1}", data.co2, data.temp);
    }
}
