use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Macropad {
    pub device: Device,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub orientation: String,
    pub rows: u8,
    pub cols: u8,
    pub knobs: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub buttons: Vec<Vec<String>>,
    pub knobs: Vec<Knob>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Knob {
    pub ccw: String,
    pub click: String,
    pub cw: String,
}

use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use std::fs::File;
use std::str::FromStr;

use crate::config::Orientation;
use crate::consts;
use crate::keyboard::{MediaCode, Modifier, WellKnownCode};

pub struct Mapping {}

impl Mapping {
    pub fn read() -> Macropad {
        // read configuration
        let cfg_file = "./mapping.ron";
        println!("configuration file: {}", cfg_file);
        let f = File::open(cfg_file).expect("Failed opening file");
        let config: Macropad = match from_reader(f) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        };
        config
    }

    pub fn print(config: Macropad) {
        let pretty = PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(false);

        let s = to_string_pretty(&config, pretty).expect("Serialization failed");
        println!("------------------------------");
        println!("{s}");
    }

    pub fn validate() -> anyhow::Result<()> {
        // check layers
        let cfg = Self::read();

        // check orientation
        Orientation::from_str(&Self::uppercase_first(&cfg.device.orientation))?;

        if cfg.layers.len() == 0 || cfg.layers.len() > 3 {
            return Err(anyhow!("number of layers must be > 0 and < 4"));
        }

        // check rows/cols/knobs
        for (i, layer) in cfg.layers.iter().enumerate() {
            // row check
            if layer.buttons.len() != cfg.device.rows.into() {
                return Err(anyhow!(
                    "number of rows mismatch at layer {}. Expected {} rows found {}",
                    i + 1,
                    cfg.device.rows,
                    layer.buttons.len(),
                ));
            }

            // column check
            for (j, btn_mapping) in layer.buttons.iter().enumerate() {
                if btn_mapping.len() != cfg.device.cols.into() {
                    return Err(anyhow!(
                        "number of colums mismatch at layer {} row {}. Expected {} columns found {}",
                        i + 1,
                        j + 1,
                        cfg.device.cols,
                        btn_mapping.len()
                    ));
                }

                // check the individual button
                for btn in btn_mapping {
                    println!("btn: {}", btn);
                    Self::validate_key_mapping(btn.to_string())?;
                }
            }

            // knob check
            if layer.knobs.len() != cfg.device.knobs.into() {
                return Err(anyhow!(
                    "number of knobs mismatch at layer {}. Expected {} knobs found {}",
                    i + 1,
                    cfg.device.knobs,
                    layer.knobs.len(),
                ));
            }
        }

        Ok(())
    }

    fn validate_key_mapping(key: String) -> Result<()> {
        // ensure we don't go over max
        let keys: Vec<_> = key.split('-').collect();
        if keys.len() > consts::MAX_KEY_PRESSES {
            return Err(anyhow!(
                "One key can be mapped to a maximum of {} key presses",
                consts::MAX_KEY_PRESSES
            ));
        }

        // check individual keys
        for k in keys {
            let da_key = Self::uppercase_first(k);
            println!("da_key: {da_key}");
            // could be media, control, or regular key
            let mut found = false;
            for i in 0..3 {
                match i {
                    0 => {
                        found = Self::is_control_key(&da_key);
                    }
                    1 => {
                        found = Self::is_media_key(&da_key);
                    }
                    2 => {
                        found = Self::is_regular_key(&da_key);
                    }
                    _ => {
                        panic!("unaccounted key test")
                    }
                }
            }
            if !found {
                return Err(anyhow!("unknown key - {}", k));
            }
        }
        Ok(())
    }

    fn uppercase_first(data: &str) -> String {
        let mut result = String::new();
        let mut first = true;
        for value in data.chars() {
            if first {
                result.push(value.to_ascii_uppercase());
                first = false;
            } else {
                result.push(value);
            }
        }
        result
    }

    fn is_control_key(keystr: &String) -> bool {
        println!("******** {keystr}");
        let ck = Modifier::from_str(&keystr);
        if ck.is_ok() {
            return true;
        }
        false
    }

    fn is_media_key(keystr: &String) -> bool {
        let mk = MediaCode::from_str(&keystr);
        if mk.is_ok() {
            return true;
        }
        false
    }

    fn is_regular_key(keystr: &String) -> bool {
        let rk = WellKnownCode::from_str(&keystr);
        if rk.is_ok() {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {

    use crate::mapping::Mapping;

    #[test]
    fn mapping_read() {
        Mapping::read();
    }

    #[test]
    fn mapping_print() {
        Mapping::print(Mapping::read());
    }

    #[test]
    fn mapping_validate() -> anyhow::Result<()> {
        Mapping::validate()?;
        Ok(())
    }
}
