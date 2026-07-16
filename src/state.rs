//! Device state representation.
//!
//! [`State`] holds the complete snapshot of a GREE device's current
//! settings. It is populated from device responses and can be diffed
//! against a previous snapshot to compute property changes.

use serde::{Deserialize, Serialize};

use crate::models::{FanSpeed, Mode, SwingHorizontal, SwingVertical, TemperatureUnit};

/// Represents the complete state of a GREE air conditioner.
///
/// All fields are populated from the device's response to a status query.
/// Use [`Client::refresh`](crate::Client::refresh) to update this state from
/// the device, and [`Client::state`](crate::Client::state) to read it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    /// Whether the unit is powered on.
    pub power: bool,
    /// Current operating mode.
    pub mode: Mode,
    /// Target (set) temperature in degrees Celsius or Fahrenheit.
    pub target_temperature: u8,
    /// Current room temperature reported by the device, if available.
    pub current_temperature: Option<u8>,
    /// Temperature unit configured on the device.
    pub temperature_unit: TemperatureUnit,
    /// Current fan speed setting.
    pub fan_speed: FanSpeed,
    /// Vertical swing / louver position.
    pub swing_vertical: SwingVertical,
    /// Horizontal swing / louver position.
    pub swing_horizontal: SwingHorizontal,
    /// Whether turbo mode is active.
    pub turbo: bool,
    /// Whether quiet mode is active.
    pub quiet: bool,
    /// Whether sleep mode is active.
    pub sleep: bool,
    /// Whether steady heat (8°C maintenance) mode is active.
    pub steady_heat: bool,
    /// Whether power save mode is active.
    pub power_save: bool,
    /// Whether fresh air circulation is enabled.
    pub fresh_air: bool,
    /// Whether X-Fan (coil drying) is enabled.
    pub xfan: bool,
    /// Whether the anion generator / ionizer is enabled.
    pub anion: bool,
    /// Whether the front panel light is on.
    pub light: bool,
    /// Target relative humidity percentage (dehumidifier mode).
    pub target_humidity: Option<u8>,
    /// Current relative humidity percentage reported by the device.
    pub current_humidity: Option<u8>,
    /// Whether the filter needs cleaning.
    pub clean_filter: bool,
    /// Whether the water tank is full (dehumidifier model).
    pub water_full: bool,
    /// Firmware version string, if available.
    pub firmware_version: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            power: false,
            mode: Mode::Auto,
            target_temperature: 24,
            current_temperature: None,
            temperature_unit: TemperatureUnit::Celsius,
            fan_speed: FanSpeed::Auto,
            swing_vertical: SwingVertical::Default,
            swing_horizontal: SwingHorizontal::Default,
            turbo: false,
            quiet: false,
            sleep: false,
            steady_heat: false,
            power_save: false,
            fresh_air: false,
            xfan: false,
            anion: false,
            light: false,
            target_humidity: None,
            current_humidity: None,
            clean_filter: false,
            water_full: false,
            firmware_version: None,
        }
    }
}

impl State {
    /// Updates the state from a raw properties map returned by the device.
    ///
    /// This handles the conversion of integer values into typed enum variants.
    pub fn merge_from_properties(
        &mut self,
        props: &std::collections::HashMap<String, serde_json::Value>,
        hid: Option<&str>,
    ) {
        use serde_json::Value;

        if let Some(v) = props.get("Pow") {
            self.power = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Mod")
            && let Some(n) = v.as_u64()
            && let Ok(mode) = Mode::try_from(n as u8)
        {
            self.mode = mode;
        }
        if let Some(v) = props.get("SetTem") {
            self.target_temperature = v
                .as_u64()
                .map(|n| n as u8)
                .unwrap_or(self.target_temperature);
        }
        if let Some(v) = props.get("TemSen") {
            let raw = v.as_u64().map(|n| n as i16).unwrap_or(0);
            let bit = props.get("TemRec").and_then(Value::as_u64).unwrap_or(0);
            self.current_temperature =
                Some(Self::calc_current_temp(raw, bit, self.temperature_unit));
        }
        if let Some(v) = props.get("TemUn")
            && let Some(n) = v.as_u64()
            && let Ok(unit) = TemperatureUnit::try_from(n as u8)
        {
            self.temperature_unit = unit;
        }
        if let Some(v) = props.get("WdSpd")
            && let Some(n) = v.as_u64()
            && let Ok(fs) = FanSpeed::try_from(n as u8)
        {
            self.fan_speed = fs;
        }
        if let Some(v) = props.get("SwUpDn")
            && let Some(n) = v.as_u64()
            && let Ok(sw) = SwingVertical::try_from(n as u8)
        {
            self.swing_vertical = sw;
        }
        if let Some(v) = props.get("SwingLfRig")
            && let Some(n) = v.as_u64()
            && let Ok(sw) = SwingHorizontal::try_from(n as u8)
        {
            self.swing_horizontal = sw;
        }
        if let Some(v) = props.get("Tur") {
            self.turbo = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Quiet") {
            self.quiet = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("SwhSlp") {
            self.sleep = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("StHt") {
            self.steady_heat = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("SvSt") {
            self.power_save = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Air") {
            self.fresh_air = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Blo") {
            self.xfan = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Health") {
            self.anion = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Lig") {
            self.light = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("Dwet") {
            self.target_humidity = v.as_u64().map(|n| (n as u8) * 5 + 15);
        }
        if let Some(v) = props.get("DwatSen") {
            self.current_humidity = v.as_u64().map(|n| n as u8);
        }
        if let Some(v) = props.get("Dfltr") {
            self.clean_filter = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("DwatFul") {
            self.water_full = v.as_u64().map(|n| n > 0).unwrap_or(false);
        }
        if let Some(v) = props.get("hid") {
            self.firmware_version = v.as_str().map(|s| s.to_string());
        } else if let Some(h) = hid {
            self.firmware_version = Some(h.to_string());
        }
    }

    fn calc_current_temp(raw: i16, _bit: u64, unit: TemperatureUnit) -> u8 {
        let temp = match unit {
            TemperatureUnit::Celsius if raw > 40 => raw - 40,
            _ => raw,
        };
        temp.clamp(0, 60) as u8
    }

    /// Returns a list of (property_name, value) pairs for properties that
    /// differ from a previous state snapshot. Used for partial state updates.
    pub fn changed_properties(&self, previous: &State) -> Vec<(&'static str, serde_json::Value)> {
        let mut props = Vec::new();

        if self.power != previous.power {
            props.push(("Pow", serde_json::json!(self.power as u8)));
        }
        if self.mode != previous.mode {
            props.push(("Mod", serde_json::json!(self.mode as u8)));
        }
        if self.target_temperature != previous.target_temperature {
            props.push(("SetTem", serde_json::json!(self.target_temperature)));
        }
        if self.fan_speed != previous.fan_speed {
            props.push(("WdSpd", serde_json::json!(self.fan_speed as u8)));
        }
        if self.swing_vertical != previous.swing_vertical {
            props.push(("SwUpDn", serde_json::json!(self.swing_vertical as u8)));
        }
        if self.swing_horizontal != previous.swing_horizontal {
            props.push(("SwingLfRig", serde_json::json!(self.swing_horizontal as u8)));
        }
        if self.turbo != previous.turbo {
            props.push(("Tur", serde_json::json!(self.turbo as u8)));
        }
        if self.quiet != previous.quiet {
            props.push((
                "Quiet",
                serde_json::json!(if self.quiet { 2u8 } else { 0u8 }),
            ));
        }
        if self.sleep != previous.sleep {
            props.push(("SwhSlp", serde_json::json!(self.sleep as u8)));
        }
        if self.steady_heat != previous.steady_heat {
            props.push(("StHt", serde_json::json!(self.steady_heat as u8)));
        }
        if self.power_save != previous.power_save {
            props.push(("SvSt", serde_json::json!(self.power_save as u8)));
        }
        if self.fresh_air != previous.fresh_air {
            props.push(("Air", serde_json::json!(self.fresh_air as u8)));
        }
        if self.xfan != previous.xfan {
            props.push(("Blo", serde_json::json!(self.xfan as u8)));
        }
        if self.anion != previous.anion {
            props.push(("Health", serde_json::json!(self.anion as u8)));
        }
        if self.light != previous.light {
            props.push(("Lig", serde_json::json!(self.light as u8)));
        }

        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = State::default();
        assert!(!state.power);
        assert_eq!(state.mode, Mode::Auto);
        assert_eq!(state.target_temperature, 24);
        assert_eq!(state.fan_speed, FanSpeed::Auto);
    }

    #[test]
    fn test_merge_from_properties_power_on() {
        let mut state = State::default();
        let mut props = std::collections::HashMap::new();
        props.insert("Pow".to_string(), serde_json::json!(1));
        state.merge_from_properties(&props, None);
        assert!(state.power);
    }

    #[test]
    fn test_merge_from_properties_mode_cool() {
        let mut state = State::default();
        let mut props = std::collections::HashMap::new();
        props.insert("Mod".to_string(), serde_json::json!(1));
        state.merge_from_properties(&props, None);
        assert_eq!(state.mode, Mode::Cool);
    }

    #[test]
    fn test_merge_from_properties_temp() {
        let mut state = State::default();
        let mut props = std::collections::HashMap::new();
        props.insert("SetTem".to_string(), serde_json::json!(25));
        props.insert("TemSen".to_string(), serde_json::json!(65));
        state.merge_from_properties(&props, None);
        assert_eq!(state.target_temperature, 25);
        assert_eq!(state.current_temperature, Some(25));
    }

    #[test]
    fn test_changed_properties() {
        let prev = State::default();
        let curr = State {
            power: true,
            mode: Mode::Cool,
            ..Default::default()
        };
        let changes = curr.changed_properties(&prev);
        assert!(!changes.is_empty());
        assert!(changes.iter().any(|(k, _)| *k == "Pow"));
        assert!(changes.iter().any(|(k, _)| *k == "Mod"));
    }

    #[test]
    fn test_no_changed_properties() {
        let prev = State::default();
        let curr = State::default();
        let changes = curr.changed_properties(&prev);
        assert!(changes.is_empty());
    }
}
