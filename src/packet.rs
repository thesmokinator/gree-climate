#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crypto::Cipher;
use crate::error::Error;

pub const GREE_PORT: u16 = 7000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    #[serde(default = "default_cid")]
    pub cid: String,
    #[serde(default)]
    pub i: u8,
    pub t: String,
    #[serde(default)]
    pub uid: u64,
    #[serde(default)]
    pub tcid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

fn default_cid() -> String {
    "app".to_string()
}

impl Packet {
    pub fn scan() -> Self {
        Self {
            cid: "app".to_string(),
            i: 1,
            t: "scan".to_string(),
            uid: 0,
            tcid: String::new(),
            pack: None,
            tag: None,
        }
    }

    pub fn bind(mac: &str) -> Self {
        let inner = serde_json::json!({
            "t": "bind",
            "mac": mac,
            "uid": 0,
        });
        Self {
            cid: "app".to_string(),
            i: 1,
            t: "pack".to_string(),
            uid: 0,
            tcid: mac.to_string(),
            pack: Some(inner),
            tag: None,
        }
    }

    pub fn status(mac: &str, cols: Vec<&str>) -> Self {
        let inner = serde_json::json!({
            "t": "status",
            "mac": mac,
            "cols": cols,
        });
        Self {
            cid: "app".to_string(),
            i: 0,
            t: "pack".to_string(),
            uid: 0,
            tcid: mac.to_string(),
            pack: Some(inner),
            tag: None,
        }
    }

    pub fn command(mac: &str, opts: Vec<&str>, values: Vec<Value>) -> Self {
        let inner = serde_json::json!({
            "t": "cmd",
            "mac": mac,
            "opt": opts,
            "p": values,
        });
        Self {
            cid: "app".to_string(),
            i: 0,
            t: "pack".to_string(),
            uid: 0,
            tcid: mac.to_string(),
            pack: Some(inner),
            tag: None,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(self).map_err(Error::Json)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice(data).map_err(Error::Json)
    }

    pub fn encrypt_pack(&mut self, cipher: &Cipher) -> Result<(), Error> {
        if let Some(ref pack) = self.pack {
            let pack_str = serde_json::to_string(pack)?;
            let (encoded, tag) = cipher.encrypt(&pack_str)?;
            self.pack = Some(serde_json::Value::String(encoded));
            self.tag = tag;
        }
        Ok(())
    }

    pub fn decrypt_pack(&self, cipher: &Cipher) -> Result<serde_json::Value, Error> {
        let pack = self
            .pack
            .as_ref()
            .ok_or(Error::MissingPack)?;

        if let Some(obj) = pack.as_object() {
            return Ok(serde_json::Value::Object(obj.clone()));
        }

        let pack_str = pack
            .as_str()
            .ok_or(Error::InvalidPacket)?;
        let decrypted_str = cipher.decrypt(pack_str)?;
        let value: serde_json::Value = serde_json::from_str(&decrypted_str)?;
        Ok(value)
    }

    pub fn get_inner_type(&self, cipher: &Cipher) -> Result<String, Error> {
        let inner = self.decrypt_pack(cipher)?;
        inner["t"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(Error::InvalidPacket)
    }

    pub fn is_bind_ok(&self, cipher: &Cipher) -> Result<bool, Error> {
        Ok(self.get_inner_type(cipher)? == "bindok")
    }

    pub fn get_bind_key(&self, cipher: &Cipher) -> Result<String, Error> {
        let inner = self.decrypt_pack(cipher)?;
        inner["key"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                Error::Authentication("no key in bindok response".into())
            })
    }
}

pub fn parse_packet(raw: &[u8]) -> Result<Packet, Error> {
    Packet::from_bytes(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_packet() {
        let packet = Packet::scan();
        assert_eq!(packet.t, "scan");
        assert_eq!(packet.cid, "app");
        assert_eq!(packet.i, 1);
        let json = serde_json::to_string(&packet).unwrap();
        assert!(json.contains("\"t\":\"scan\""));
    }

    #[test]
    fn test_bind_packet() {
        let packet = Packet::bind("aabbcc112233");
        assert_eq!(packet.tcid, "aabbcc112233");

        let inner = packet.pack.as_ref().unwrap();
        assert_eq!(inner["t"], "bind");
        assert_eq!(inner["mac"], "aabbcc112233");
    }

    #[test]
    fn test_status_packet() {
        let packet = Packet::status("aabbcc112233", vec!["Pow", "Mod"]);
        let inner = packet.pack.as_ref().unwrap();
        assert_eq!(inner["t"], "status");
        assert_eq!(inner["cols"][0], "Pow");
    }

    #[test]
    fn test_command_packet() {
        let packet = Packet::command(
            "aabbcc112233",
            vec!["Pow", "SetTem"],
            vec![serde_json::json!(1), serde_json::json!(23)],
        );
        let inner = packet.pack.as_ref().unwrap();
        assert_eq!(inner["t"], "cmd");
        assert_eq!(inner["opt"][0], "Pow");
        assert_eq!(inner["p"][0], 1);
    }

    #[test]
    fn test_serialize_deserialize() {
        let packet = Packet::scan();
        let bytes = packet.to_bytes().unwrap();
        let parsed = parse_packet(&bytes).unwrap();
        assert_eq!(parsed.t, "scan");
    }

    #[test]
    fn test_scan_packet_has_no_tag() {
        let packet = Packet::scan();
        assert!(packet.tag.is_none());
        assert!(packet.pack.is_none());
    }

    #[test]
    fn test_roundtrip_encrypt_decrypt_pack() {
        use crate::crypto::CipherV1;
        let cipher = CipherV1::new();
        let mut packet = Packet::bind("aabbcc112233");
        let cipher_enum = crate::crypto::Cipher::V1(cipher);
        packet.encrypt_pack(&cipher_enum).unwrap();
        assert!(packet.pack.as_ref().unwrap().is_string());
        let inner = packet.decrypt_pack(&cipher_enum).unwrap();
        assert_eq!(inner["t"], "bind");
        assert_eq!(inner["mac"], "aabbcc112233");
    }
}
