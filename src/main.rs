extern crate hidapi;
use hidapi::HidApi;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let vendor_id = 0x04D9;
    let product_id = 0xA052;
    let key = [0x86, 0x41, 0xc9, 0xa8, 0x7f, 0x41, 0x3c, 0xac];
    let offset = [0x48,  0x74,  0x65,  0x6D,  0x70,  0x39,  0x39,  0x65];
    let shuffle = [2, 4, 0, 7, 1, 6, 5, 3];

    loop {
        match HidApi::new() {
            Ok(api) => {
                let device = api.open(vendor_id, product_id).expect("CO2-mini is not inserted!!");
                device.send_feature_report(&key).unwrap();
                let mut buf = [0u8; 8];
                let res = device.read(&mut buf);
                match res {
                    Ok(_) => {
                        //println!("{:?}", &buf[..result]);
                        let phase1 = shuffle.iter()
                            .map(|x| buf[*x] as u32)
                            .collect::<Vec<u32>>();
                        //println!("{:?}", phase1);
                        let phase2 = (0..=7).map(|x| phase1[x] ^ key[x] as u32)
                            .collect::<Vec<u32>>();
                        //println!("{:?}", phase2);
                        let phase3 = (0..=7).map(|x| (phase2[x] >> 3) | (phase2[ ((x as i16 -1+8)%8) as usize ] << 5)  & 0xff)
                            .collect::<Vec<u32>>();
                        //println!("{:?}", phase3);
                        let ctmp   = (0..=7).map(|i| ((offset[i] >> 4) | offset[i] << 4 )  & 0xff)
                            .collect::<Vec<u32>>();
                        let result = (0..7).map(|i| ((0x100u32 + phase3[i] - ctmp[i]) & 0xff) as u8 )
                            .collect::<Vec<u8>>();
                        //println!("{:?}", result);
                        let val = ((result[1] as u16) << 8) | result[2] as u16;
                        let op = result[0];
                        match op {
                            0x50 => println!("co2: {}", val),
                            0x42 => println!("temp: {}", val as f32 / 16.0 - 273.15),
                            _ => (),
                        }

                    },
                    Err(_) => {
                        println!("err");
                    }
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            },
        }
        let time = Duration::from_millis(1000);
        sleep(time);
    }
}
