use std::collections::HashMap;

use gree_climate::{
    commands::Commands,
    crypto::Cipher,
    device::DeviceInfo,
    models::{FanSpeed, Mode, SwingHorizontal, SwingVertical},
    state::State,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_packet_flow_v1() {
        let cipher = Cipher::v1();
        let mac = "aabbcc112233";

        let mut bind_pkt = Commands::bind_packet(mac);
        bind_pkt.encrypt_pack(&cipher).unwrap();

        let inner = bind_pkt.decrypt_pack(&cipher).unwrap();
        assert_eq!(inner["t"], "bind");
        assert_eq!(inner["mac"], mac);
    }

    #[test]
    fn test_full_packet_flow_v2() {
        let cipher = Cipher::v2();
        let mac = "aabbcc112233";

        let mut bind_pkt = Commands::bind_packet(mac);
        bind_pkt.encrypt_pack(&cipher).unwrap();

        let inner = bind_pkt.decrypt_pack(&cipher).unwrap();
        assert_eq!(inner["t"], "bind");
        assert_eq!(inner["mac"], mac);
    }

    #[test]
    fn test_status_packet_encryption_roundtrip() {
        let cipher = Cipher::v1();
        let mac = "device-mac-01";

        let mut pkt = Commands::status_packet(mac, true);
        pkt.encrypt_pack(&cipher).unwrap();
        let inner = pkt.decrypt_pack(&cipher).unwrap();

        assert_eq!(inner["t"], "status");
        assert_eq!(inner["mac"], mac);
        let cols = inner["cols"].as_array().unwrap();
        assert!(cols.contains(&serde_json::json!("Pow")));
        assert!(cols.contains(&serde_json::json!("hid")));
    }

    #[test]
    fn test_command_packet_encryption_roundtrip() {
        let cipher = Cipher::v1();
        let mac = "device-mac-02";
        let opts = vec!["Pow", "SetTem"];
        let values = vec![serde_json::json!(1), serde_json::json!(23)];

        let pkt = Commands::cmd_packet(mac, opts, values);
        let mut encrypted = pkt.clone();
        encrypted.encrypt_pack(&cipher).unwrap();

        let inner = encrypted.decrypt_pack(&cipher).unwrap();
        assert_eq!(inner["t"], "cmd");
        assert_eq!(inner["opt"][0], "Pow");
        assert_eq!(inner["p"][0], 1);
        assert_eq!(inner["opt"][1], "SetTem");
        assert_eq!(inner["p"][1], 23);
    }

    #[test]
    fn test_state_roundtrip() {
        let mut state = State::default();
        let mut props = HashMap::new();
        props.insert("Pow".to_string(), serde_json::json!(1));
        props.insert("Mod".to_string(), serde_json::json!(1)); // Cool
        props.insert("SetTem".to_string(), serde_json::json!(22));
        props.insert("WdSpd".to_string(), serde_json::json!(3)); // Medium
        props.insert("Tur".to_string(), serde_json::json!(1));
        props.insert("SwhSlp".to_string(), serde_json::json!(1));
        props.insert("Lig".to_string(), serde_json::json!(1));
        props.insert("SwUpDn".to_string(), serde_json::json!(1)); // FullSwing
        props.insert("SwingLfRig".to_string(), serde_json::json!(3)); // LeftCenter

        state.merge_from_properties(&props, None);

        assert!(state.power);
        assert_eq!(state.mode, Mode::Cool);
        assert_eq!(state.target_temperature, 22);
        assert_eq!(state.fan_speed, FanSpeed::Medium);
        assert!(state.turbo);
        assert!(state.sleep);
        assert!(state.light);
        assert_eq!(state.swing_vertical, SwingVertical::FullSwing);
        assert_eq!(state.swing_horizontal, SwingHorizontal::LeftCenter);
    }

    #[test]
    fn test_device_info_equality() {
        let d1 = DeviceInfo::new(
            "192.168.1.100".into(),
            7000,
            "aabbcc112233".into(),
            "Room 1".into(),
        );
        let d2 = DeviceInfo::new(
            "192.168.1.101".into(),
            7000,
            "aabbcc112233".into(),
            "Room 1".into(),
        );

        assert_eq!(d1, d2);
        assert_eq!(d1.mac, d2.mac);
        assert_ne!(d1.ip, d2.ip);
    }

    #[test]
    fn test_cipher_key_management() {
        let custom_key = "abcd1234abcd1234";

        let mut cipher = Cipher::v1();
        assert_eq!(cipher.key(), "a3K8Bx%2r8Y7#xDh");

        cipher.set_key(custom_key);
        assert_eq!(cipher.key(), custom_key);

        let data = r#"{"test":"value"}"#;

        let (encrypted, _) = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_all_commands_roundtrip() {
        type CmdFn = fn(&str) -> gree_climate::packet::Packet;
        let test_cases: Vec<(&str, CmdFn)> = vec![
            ("set_power", |m| Commands::set_power(m, true)),
            ("set_mode", |m| Commands::set_mode(m, Mode::Heat)),
            ("set_temperature", |m| Commands::set_temperature(m, 25)),
            ("set_fan_speed", |m| {
                Commands::set_fan_speed(m, FanSpeed::High)
            }),
            ("set_swing_vertical", |m| {
                Commands::set_swing_vertical(m, SwingVertical::FullSwing)
            }),
            ("set_swing_horizontal", |m| {
                Commands::set_swing_horizontal(m, SwingHorizontal::FullSwing)
            }),
            ("set_turbo", |m| Commands::set_turbo(m, true)),
            ("set_quiet", |m| Commands::set_quiet(m, true)),
            ("set_sleep", |m| Commands::set_sleep(m, false)),
            ("set_steady_heat", |m| Commands::set_steady_heat(m, true)),
            ("set_power_save", |m| Commands::set_power_save(m, false)),
            ("set_fresh_air", |m| Commands::set_fresh_air(m, true)),
            ("set_xfan", |m| Commands::set_xfan(m, true)),
            ("set_anion", |m| Commands::set_anion(m, false)),
            ("set_light", |m| Commands::set_light(m, true)),
        ];

        let cipher = Cipher::v1();
        let mac = "test-mac-00";

        for (name, cmd_fn) in test_cases {
            let mut pkt = cmd_fn(mac);
            pkt.encrypt_pack(&cipher).unwrap();
            let inner = pkt.decrypt_pack(&cipher).unwrap();
            assert_eq!(inner["t"], "cmd", "Command {name} has incorrect inner type");
            assert!(
                inner["opt"].as_array().is_some(),
                "Command {name} has no opt array"
            );
        }
    }

    #[test]
    fn test_state_changed_properties_all() {
        let prev = State::default();
        let curr = State {
            power: true,
            mode: Mode::Heat,
            target_temperature: 28,
            fan_speed: FanSpeed::High,
            swing_vertical: SwingVertical::FullSwing,
            swing_horizontal: SwingHorizontal::Right,
            turbo: true,
            quiet: true,
            sleep: true,
            steady_heat: true,
            power_save: true,
            fresh_air: true,
            xfan: true,
            anion: true,
            light: true,
            ..Default::default()
        };

        let changes = curr.changed_properties(&prev);
        assert!(!changes.is_empty());
        assert!(changes.len() >= 14);
    }

    #[test]
    fn test_error_display() {
        let err = gree_climate::Error::Timeout;
        assert!(format!("{err}").contains("time"));
        assert_eq!(format!("{err}"), "device communication timed out");

        let err = gree_climate::Error::DeviceNotBound;
        assert!(format!("{err}").contains("not bound"), "Error: {err}");
    }
}
