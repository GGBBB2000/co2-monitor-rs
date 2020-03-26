use hidapi::HidApi;

pub struct Co2Monitor {
    device: hidapi::HidDevice,
}

impl Co2Monitor {
    /// - Feature Report用の値
    const KEY: [u8; 8] = [0x86, 0x41, 0xc9, 0xa8, 0x7f, 0x41, 0x3c, 0xac];
    /// - Decrypt用の定数
    const OFFSET: [u32; 8] = [0x48, 0x74, 0x65, 0x6D, 0x70, 0x39, 0x39, 0x65]; // "Htemp99e"
    /// - Decrypt用の定数
    const SHUFFLE: [u32; 8] = [2, 4, 0, 7, 1, 6, 5, 3];
    /// - Vendor Id 
    const VID: u16 = 0x04D9;
    /// - Product Id 
    const PID: u16 = 0xA052;

    /// - sudo権限が必要
    pub fn new() -> Self {
        let api = HidApi::new().unwrap();
        Co2Monitor {
            device: api.open(Self::VID, Self::PID).expect("CO2-mini is not detected!! or Permission Denied!!"),
        }
    }

    /// - CO2-miniにfeature reportを送る
    pub fn init(&mut self) {
        self.device.send_feature_report(&Self::KEY).unwrap();
    }

    pub fn read(&self) -> Result<(u16, f32), hidapi::HidError> {
        let mut buf = [0u8; 8];
        let mut co2_checked = false;
        let mut temp_checked = false;
        let mut co2_val: u16 = 0;
        let mut temp_val: f32 = 0f32;
        while !(co2_checked && temp_checked) {
            self.device.read(&mut buf)?;
            let result = Self::decrypt(buf);
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
        Ok((co2_val, temp_val))
    }

    fn decrypt(buf: [u8; 8]) -> Vec<u8> {
        let phase1 = &Self::SHUFFLE
            .iter()
            .map(|&x| buf[x as usize] as u32)
            .collect::<Vec<_>>();
        //println!("{:?}", phase1);
        let phase2 = phase1
            .iter()
            .zip(Self::KEY.iter())
            .map(|(&p1, &k)| p1 ^ k as u32)
            .collect::<Vec<_>>();
        //println!("{:?}", phase2);
        let phase3 = (0..=7)
            .map(|x| (phase2[x] >> 3) | (phase2[(x + 7) % 8] << 5) & 0xff)
            .collect::<Vec<_>>();
        //println!("{:?}", phase3);
        let ctmp = Self::OFFSET
            .iter()
            .map(|o| ((o >> 4) | o << 4) & 0xff)
            .collect::<Vec<_>>();
        phase3
            .into_iter()
            .zip(ctmp.into_iter())
            .map(|(p3, c)| ((0x100u32 + p3 - c) & 0xff) as u8)
            .collect::<Vec<_>>()
    }
}
