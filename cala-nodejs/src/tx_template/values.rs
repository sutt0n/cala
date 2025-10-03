use crate::query::CursorToken;

#[napi(object)]
pub struct NewParamDefinitionValues {
  pub name: String,
  pub r#type: ParamDataTypeValues,
  pub default: Option<String>,
  pub description: Option<String>,
}

#[napi(object)]
pub struct NewTxTemplateEntryValues {
  pub entry_type: String,
  pub account_id: String,
  pub layer: String,
  pub direction: String,
  pub units: String,
  pub currency: String,
  pub description: Option<String>,
  pub metadata: Option<String>,
}

#[napi(object)]
pub struct NewTxTemplateValues {
  pub id: Option<String>,
  pub code: String,
  pub external_id: Option<String>,
  pub description: Option<String>,
  pub params: Option<Vec<NewParamDefinitionValues>>,
  pub entries: Vec<NewTxTemplateEntryValues>,
  pub metadata: Option<serde_json::Value>,
  pub transaction: Option<NewTxTemplateTransactionValues>,
}

#[napi(object)]
pub struct NewTxTemplateTransactionValues {
  pub effective: String,
  pub journal_id: String,
  pub correlation_id: Option<String>,
  pub external_id: Option<String>,
  pub description: Option<String>,
  pub metadata: Option<String>,
}

#[napi(object)]
pub struct TxTemplateValues {
  pub id: String,
  pub code: String,
  pub version: u32,
  pub metadata: Option<serde_json::Value>,
  pub description: Option<String>,
}

#[napi]
pub enum ParamDataTypeValues {
  String,
  Integer,
  Decimal,
  Boolean,
  Uuid,
  Date,
  Timestamp,
  Json,
}

#[napi(object)]
pub struct PaginatedTxTemplates {
  pub tx_templates: Vec<TxTemplateValues>,
  pub has_next_page: bool,
  pub end_cursor: Option<CursorToken>,
}

impl From<cala_ledger::tx_template::TxTemplatesByCodeCursor> for CursorToken {
  fn from(cursor: cala_ledger::tx_template::TxTemplatesByCodeCursor) -> Self {
    use base64::{engine::general_purpose, Engine as _};
    let json = serde_json::to_string(&cursor).expect("could not serialize token");
    let token: String = general_purpose::STANDARD_NO_PAD.encode(json.as_bytes());
    CursorToken { token }
  }
}
impl TryFrom<CursorToken> for cala_ledger::tx_template::TxTemplatesByCodeCursor {
  type Error = napi::Error;

  fn try_from(token: CursorToken) -> Result<Self, Self::Error> {
    use base64::{engine::general_purpose, Engine as _};
    let json_bytes = general_purpose::STANDARD_NO_PAD
      .decode(token.token)
      .map_err(crate::generic_napi_error)?;
    let json = String::from_utf8(json_bytes).map_err(crate::generic_napi_error)?;
    serde_json::from_str(&json).map_err(crate::generic_napi_error)
  }
}

impl From<&cala_ledger::tx_template::TxTemplate> for TxTemplateValues {
  fn from(template: &cala_ledger::tx_template::TxTemplate) -> Self {
    let values = template.values().clone();
    Self {
      id: values.id.to_string(),
      code: values.code.to_string(),
      description: values.description,
      metadata: values.metadata,
      version: values.version,
    }
  }
}

impl From<cala_ledger::tx_template::TxTemplate> for TxTemplateValues {
  fn from(template: cala_ledger::tx_template::TxTemplate) -> Self {
    let values = template.into_values();
    Self {
      id: values.id.to_string(),
      code: values.code.to_string(),
      description: values.description,
      metadata: values.metadata,
      version: values.version,
    }
  }
}
