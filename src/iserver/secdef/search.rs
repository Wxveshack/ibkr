//! `GET /iserver/secdef/search` — search contracts by symbol or company name.
//! Docs: <https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/#search-symbol-contract>

use crate::Endpoint;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Request {
    pub symbol: String,
    pub name: Option<bool>,
    pub sec_type: Option<SecType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecType {
    Stk,
    Ind,
    Bond,
}

impl SecType {
    pub fn as_str(self) -> &'static str {
        match self {
            SecType::Stk => "STK",
            SecType::Ind => "IND",
            SecType::Bond => "BOND",
        }
    }
}

impl Endpoint for Request {
    type Response = Vec<Contract>;
    const METHOD: reqwest::Method = reqwest::Method::GET;

    fn path(&self) -> String {
        "/iserver/secdef/search".to_string()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = vec![("symbol".to_string(), self.symbol.clone())];
        if let Some(name) = self.name {
            q.push(("name".to_string(), name.to_string()));
        }
        if let Some(sec_type) = self.sec_type {
            q.push(("secType".to_string(), sec_type.as_str().to_string()));
        }
        q
    }
}

/// One matched contract. Fields are optional + `serde(default)` and unknown fields
/// ignored; bond results populate [`Self::issuers`]/[`Self::bond_id`] and leave the
/// company fields null.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Contract {
    pub conid: Option<String>,
    pub company_header: Option<String>,
    pub company_name: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    /// Comma-separated secTypes barred from trading, e.g. `CFD,IOPT` — not a bool.
    pub restricted: Option<String>,
    pub sec_type: Option<String>,
    pub sections: Vec<Section>,
    pub issuers: Vec<Issuer>,
    /// Wire key is lowercase `bondid`, outside the crate-wide camelCase rename.
    #[serde(rename = "bondid")]
    pub bond_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Section {
    pub sec_type: Option<String>,
    /// Semicolon-separated, e.g. `JANYY;FEBYY;MARYY`.
    pub months: Option<String>,
    pub symbol: Option<String>,
    /// Semicolon-separated, e.g. `EXCH;EXCH;EXCH`.
    pub exchange: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Issuer {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Official example response from the docs.
    const SAMPLE: &str = r#"[
        {
            "conid":"43645865",
            "companyHeader":"IBKR INTERACTIVE BROKERS GRO-CL A (NASDAQ) ",
            "companyName":"INTERACTIVE BROKERS GRO-CL A (NASDAQ)",
            "symbol":"IBKR",
            "description":null,
            "restricted":null,
            "sections":[],
            "secType":"STK"
        }
    ]"#;

    #[test]
    fn decodes_official_sample() {
        let resp: Vec<Contract> = serde_json::from_str(SAMPLE).expect("decode sample");
        assert_eq!(resp.len(), 1);
        let c = &resp[0];
        assert_eq!(c.conid.as_deref(), Some("43645865"));
        assert_eq!(c.symbol.as_deref(), Some("IBKR"));
        assert_eq!(c.sec_type.as_deref(), Some("STK"));
        assert!(c.sections.is_empty());
    }

    #[test]
    fn ignores_unknown_fields() {
        let resp: Vec<Contract> =
            serde_json::from_str(r#"[{"conid":"265598","symbol":"AAPL","futureField":123}]"#)
                .expect("decode with unknown field");
        assert_eq!(resp[0].conid.as_deref(), Some("265598"));
        assert_eq!(resp[0].symbol.as_deref(), Some("AAPL"));
    }

    #[test]
    fn query_includes_required_and_set_optionals() {
        let req = Request {
            symbol: "IBKR".to_string(),
            name: Some(true),
            sec_type: Some(SecType::Stk),
        };
        let q = req.query();
        assert!(q.contains(&("symbol".to_string(), "IBKR".to_string())));
        assert!(q.contains(&("name".to_string(), "true".to_string())));
        assert!(q.contains(&("secType".to_string(), "STK".to_string())));

        // unset optionals are absent
        let bare = Request {
            symbol: "AAPL".to_string(),
            name: None,
            sec_type: None,
        };
        let q = bare.query();
        assert_eq!(q.len(), 1);
        assert!(!q.iter().any(|(k, _)| k == "name"));
    }
}
