use serde_json::Value;

use crate::models::{
    FanSpeed, Mode, SwingHorizontal, SwingVertical,
};
use crate::packet::Packet;

#[derive(Debug)]
pub struct Commands;

impl Commands {
    pub fn scan_packet() -> Packet {
        Packet::scan()
    }

    pub fn bind_packet(mac: &str) -> Packet {
        Packet::bind(mac)
    }

    pub fn status_packet(
        mac: &str,
        include_hid: bool,
    ) -> Packet {
        let mut cols = vec![
            "Pow", "Mod", "SetTem", "TemSen", "TemUn", "TemRec",
            "WdSpd", "SwUpDn", "SwingLfRig", "Tur", "Quiet", "SwhSlp",
            "StHt", "SvSt", "Air", "Blo", "Health", "Lig",
            "Dwet", "DwatSen", "Dfltr", "DwatFul",
        ];
        if include_hid {
            cols.push("hid");
        }
        Packet::status(mac, cols)
    }

    pub fn cmd_packet(
        mac: &str,
        opts: Vec<&str>,
        values: Vec<Value>,
    ) -> Packet {
        Packet::command(mac, opts, values)
    }

    pub fn set_power(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Pow"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_mode(mac: &str, mode: Mode) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Mod"],
            vec![Value::from(mode as u8)],
        )
    }

    pub fn set_temperature(mac: &str, temp: u8) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["SetTem"],
            vec![Value::from(temp)],
        )
    }

    pub fn set_fan_speed(mac: &str, speed: FanSpeed) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["WdSpd"],
            vec![Value::from(speed as u8)],
        )
    }

    pub fn set_swing_vertical(mac: &str, swing: SwingVertical) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["SwUpDn"],
            vec![Value::from(swing as u8)],
        )
    }

    pub fn set_swing_horizontal(
        mac: &str,
        swing: SwingHorizontal,
    ) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["SwingLfRig"],
            vec![Value::from(swing as u8)],
        )
    }

    pub fn set_turbo(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Tur"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_quiet(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Quiet"],
            vec![Value::from(if on { 2u8 } else { 0u8 })],
        )
    }

    pub fn set_sleep(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["SwhSlp"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_steady_heat(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["StHt"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_power_save(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["SvSt"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_fresh_air(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Air"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_xfan(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Blo"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_anion(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Health"],
            vec![Value::from(on as u8)],
        )
    }

    pub fn set_light(mac: &str, on: bool) -> Packet {
        Self::cmd_packet(
            mac,
            vec!["Lig"],
            vec![Value::from(on as u8)],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_packet() {
        let pkt = Commands::scan_packet();
        assert_eq!(pkt.t, "scan");
    }

    #[test]
    fn test_bind_packet() {
        let pkt = Commands::bind_packet("MAC");
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["t"], "bind");
    }

    #[test]
    fn test_status_packet() {
        let pkt = Commands::status_packet("MAC", true);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["t"], "status");
        let cols = inner["cols"].as_array().unwrap();
        assert!(cols.contains(&serde_json::json!("Pow")));
        assert!(cols.contains(&serde_json::json!("hid")));
    }

    #[test]
    fn test_set_power() {
        let pkt = Commands::set_power("MAC", true);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["opt"][0], "Pow");
        assert_eq!(inner["p"][0], 1);
    }

    #[test]
    fn test_set_mode() {
        let pkt = Commands::set_mode("MAC", Mode::Cool);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["opt"][0], "Mod");
        assert_eq!(inner["p"][0], 1);
    }

    #[test]
    fn test_set_temperature() {
        let pkt = Commands::set_temperature("MAC", 23);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["opt"][0], "SetTem");
        assert_eq!(inner["p"][0], 23);
    }

    #[test]
    fn test_set_fan_speed() {
        let pkt = Commands::set_fan_speed("MAC", FanSpeed::High);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["opt"][0], "WdSpd");
        assert_eq!(inner["p"][0], FanSpeed::High as u8);
    }

    #[test]
    fn test_set_quiet_on() {
        let pkt = Commands::set_quiet("MAC", true);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["p"][0], 2);
    }

    #[test]
    fn test_set_quiet_off() {
        let pkt = Commands::set_quiet("MAC", false);
        let inner = pkt.pack.unwrap();
        assert_eq!(inner["p"][0], 0);
    }
}
