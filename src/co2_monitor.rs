use hidapi::HidApi;

pub struct Co2Monitor {
    device: hidapi::HidDevice,
}

impl Co2Monitor {
    const KEY: [u8; 8] = [0x86, 0x41, 0xc9, 0xa8, 0x7f, 0x41, 0x3c, 0xac];
    const OFFSET: [u32; 8] = [0x48, 0x74, 0x65, 0x6D, 0x70, 0x39, 0x39, 0x65];
    const SHUFFLE: [u32; 8] = [2, 4, 0, 7, 1, 6, 5, 3];
    const VID: u16 = 0x04D9;
    const PID: u16 = 0xA052;

    pub fn new() -> Self {
        let api = HidApi::new().expect("Co2-mini is not inserted!!");
        Co2Monitor {
            device: api.open(Self::VID, Self::PID).unwrap(),
        }
    }

    pub fn init(&mut self) {
        self.device.send_feature_report(&Self::KEY).unwrap();
    }

    pub fn read(&self) -> Option<(u16, f32)> {
        let mut buf = [0u8; 8];
        let mut co2_checked = false;
        let mut temp_checked = false;
        let mut co2_val: u16 = 0;
        let mut temp_val: f32 = 0f32;
        while !(co2_checked && temp_checked) {
            let res = self.device.read(&mut buf);
            match res {
                Ok(_) => {
                    //println!("{:?}", &buf[..result]);
                    let phase1 = &Self::SHUFFLE
                        .iter()
                        .map(|x| buf[*x as usize] as u32)
                        .collect::<Vec<u32>>();
                    //println!("{:?}", phase1);
                    let phase2 = (0..=7)
                        .map(|x| phase1[x] ^ Self::KEY[x] as u32)
                        .collect::<Vec<u32>>();
                    //println!("{:?}", phase2);
                    let phase3 = (0..=7)
                        .map(|x| {
                            (phase2[x] >> 3) | (phase2[((x as i16 + 7) % 8) as usize] << 5) & 0xff
                        })
                        .collect::<Vec<u32>>();
                    //println!("{:?}", phase3);
                    let ctmp = (0..=7)
                        .map(|i| ((Self::OFFSET[i] >> 4) | Self::OFFSET[i] << 4) & 0xff)
                        .collect::<Vec<u32>>();
                    let result = (0..=7)
                        .map(|i| ((0x100u32 + phase3[i] - ctmp[i]) & 0xff) as u8)
                        .collect::<Vec<u8>>();
                    //println!("{:?}", result);
                    let val = ((result[1] as u16) << 8) | result[2] as u16;
                    let op = result[0];
                    match op {
                        0x50 => {
                            co2_checked = true;
                            co2_val = val;
                        }
                        0x42 => {
                            temp_checked = true;
                            temp_val = val as f32 / 16.0 - 273.15;
                        }
                        _ => (),
                    }
                }
                Err(_) => {
                    println!("err");
                    return None;
                }
            }
        }
        Some((co2_val, temp_val))
    }
}
