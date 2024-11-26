use cot_proto::{base::Cot, tak::detail::TakMarkerDetail};

use super::TelemMsg;

pub type CotXml = Cot<TakMarkerDetail>;

impl TelemMsg for CotXml {
    fn from_coords(lat: f64, lon: f64, alt_hae: f32) -> Self {
        let mut cot = Cot::default();
        cot.point.lat = lat;
        cot.point.lon = lon;
        cot.point.hae = alt_hae;
        cot
    }

    fn with_agent_id(mut self, agent_id: &str) -> Self {
        self.uid = agent_id.to_string();
        self.detail.contact.callsign = agent_id.to_string();
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        quick_xml::se::to_string(self).unwrap().into_bytes()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cot_serde() {
        let cot = CotXml::from_coords(45.6, -122.7, 101.0).with_agent_id("whiskey_foxtrot");
        let bytes = cot.to_bytes();
        let cot2: CotXml = quick_xml::de::from_str(&String::from_utf8(bytes).unwrap()).unwrap();
        assert_eq!(cot.point.lat, cot2.point.lat);
        assert_eq!(cot.point.lon, cot2.point.lon);
        assert_eq!(cot.point.hae, cot2.point.hae);
        assert_eq!(cot.uid, cot2.uid);
        assert_eq!(cot2.detail.contact.callsign, "whiskey_foxtrot");
    }
}
