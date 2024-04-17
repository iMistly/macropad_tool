use crate::{
    consts,
    keyboard::{LedColor, MediaCode, Modifier, WellKnownCode},
};
use anyhow::Result;
use log::debug;
use num::{FromPrimitive, ToPrimitive};
use std::str::FromStr;

pub struct Messages {}

impl Messages {
    /// Message to read the key mappings from the macropad
    ///
    /// # Arguments
    /// `keys` - The number of keys on the macropad
    /// `encoders` - The number of rotary encoders on the macropad
    /// `layers` - The layer to read the configuration for
    ///
    pub fn read_config(keys: u8, encoders: u8, layer: u8) -> Vec<u8> {
        vec![
            0x03, 0xfa, keys, encoders, layer, 0x02, 0xe0, 0xcb, 0x80, 0x00, 0xa0, 0xcc, 0x80,
            0x00, 0x7c, 0xf2, 0x02, 0x69, 0x00, 0x00, 0x00, 0x00, 0x4d, 0x00, 0x2c, 0x02, 0xa0,
            0xcc, 0x80, 0x00, 0xe8, 0x00, 0x00, 0x00, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x90, 0xcc, 0x80, 0x00, 0x20, 0xcd, 0x80, 0x00, 0xc0, 0x84, 0x26, 0x02, 0xa0,
            0x62, 0x2f, 0x02, 0xc0, 0xcc, 0x80, 0x00, 0xc7, 0xb6, 0xc2,
        ]
    }

    /// Message to read what type of macropad is connected
    ///
    pub fn device_type() -> Vec<u8> {
        vec![
            0x03, 0xfb, 0xfb, 0xfb, 0x1f, 0x02, 0x3c, 0xd0, 0x80, 0x00, 0xec, 0xcf, 0x80, 0x00,
            0xcc, 0xd2, 0x9b, 0x00, 0xf0, 0xcf, 0x80, 0x00, 0x3c, 0xd0, 0x80, 0x00, 0x56, 0x83,
            0xd2, 0x7b, 0xd0, 0x0d, 0x48, 0x00, 0x0c, 0xd0, 0x80, 0x00, 0xa8, 0x3d, 0x34, 0x02,
            0x48, 0xd0, 0x80, 0x00, 0x70, 0xf5, 0x1e, 0x62, 0x98, 0xda, 0x11, 0x62, 0x0c, 0x80,
            0x00, 0x00, 0x00, 0x82, 0x26, 0x02, 0xff, 0xff, 0xff,
        ]
    }

    /// Message sent to device when a it is done being prgrammed. This will cause the device
    /// to store they key in nvram
    ///
    pub fn end_program() -> Vec<u8> {
        vec![
            0x03, 0xfd, 0xfe, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    /// Programs the LEDs
    ///
    /// # Arguments
    /// `mode` - The mode to be set for the LEDs
    /// `keys` - The color to be set for the LEDs
    ///
    pub fn program_led(mode: u8, color: LedColor) -> Vec<u8> {
        let mut m_c = <LedColor as ToPrimitive>::to_u8(&color).unwrap();
        m_c |= mode;
        debug!("mode and code:0x{:02}", m_c);
        vec![
            0x03, 0xfe, 0xb0, 0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, m_c, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    pub fn build_key_msg(key_chord: String, layer: u8, key_pos: u8, delay: u16) -> Result<Vec<u8>> {
        let keys: Vec<_> = key_chord.split(',').collect();
        let mut msg = vec![
            0x03,
            0xfd,
            key_pos,
            layer,
            0x01,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            keys.len().try_into()?,
        ];

        let mut cnt = 0;
        for binding in &keys {
            let kc: Vec<_> = binding.split('-').collect();
            let mut m_c = 0x00u8;
            let mut wkk = 0x00;
            for (i, key) in kc.iter().enumerate() {
                println!("=> {key}");
                if let Ok(m) = Modifier::from_str(&key) {
                    let power = <Modifier as ToPrimitive>::to_u8(&m).unwrap();
                    m_c = 0u32.pow(power as u32) as u8;
                    println!("11111 - {m_c}");
                } else if let Ok(w) = WellKnownCode::from_str(&key) {
                    wkk = <WellKnownCode as ToPrimitive>::to_u8(&w).unwrap();
                    println!("22222 - {wkk}");
                } else if let Ok(a) = MediaCode::from_str(&key) {
                    //m_c = <MediaCode as ToPrimitive>::to_u8(&a).unwrap();
                    println!("33333 - FIXME: implement");
                }
                if kc.len() > 1 {
                    if i == 0 {
                        msg.extend_from_slice(&[m_c]);
                    } else {
                        msg.extend_from_slice(&[wkk]);
                        cnt += 1;
                    }
                } else {
                    msg.extend_from_slice(&[m_c, wkk]);
                    cnt += 1;
                }
            }
        }

        for _i in 0..=(consts::MAX_KEY_PRESSES - cnt) {
            msg.extend_from_slice(&[0x00, 0x00]);
        }

        // last 18 bytes are always 0
        msg.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::Messages;

    #[test]
    fn ctrl_a_ctrl_s() -> anyhow::Result<()> {
        // ctrl-a,ctrl-s
        // 03 fd 01 01 01 00 00 00     00 00 02 01 04 01 16 00   00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
        let msg = Messages::build_key_msg("ctrl-a,ctrl-s".to_string(), 1u8, 1u8, 0)?;
        println!("{:02x?}", msg);
        assert_eq!(msg.len(), 65);
        assert_eq!(msg[10], 0x02);
        assert_eq!(msg[11], 0x01);
        assert_eq!(msg[12], 0x04);
        assert_eq!(msg[13], 0x01);
        assert_eq!(msg[14], 0x16);
        Ok(())
    }

    #[test]
    fn well_known_key() -> anyhow::Result<()> {
        let msg = Messages::build_key_msg("a".to_string(), 1u8, 1u8, 0)?;
        println!("{:02x?}", msg);
        assert_eq!(msg.len(), 65);
        assert_eq!(msg[10], 0x01);
        assert_eq!(msg[11], 0x00);
        assert_eq!(msg[12], 0x04);
        Ok(())
    }
}
