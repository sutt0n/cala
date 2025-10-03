use crate::query::CursorToken;

#[napi(object)]
pub struct EntryValues {
  pub id: String,
  pub version: u32,
  pub transaction_id: String,
  pub journal_id: String,
  pub account_id: String,
  pub entry_type: String,
  pub sequence: u32,
  pub layer: String,
  pub units: String,
  pub currency: String,
  pub direction: String,
  pub description: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
pub struct PaginatedEntries {
  pub entries: Vec<EntryValues>,
  pub has_next_page: bool,
  pub end_cursor: Option<CursorToken>,
}

impl From<cala_ledger::entry::EntriesByCreatedAtCursor> for CursorToken {
  fn from(cursor: cala_ledger::entry::EntriesByCreatedAtCursor) -> Self {
    use base64::{engine::general_purpose, Engine as _};
    let json = serde_json::to_string(&cursor).expect("could not serialize token");
    let token: String = general_purpose::STANDARD_NO_PAD.encode(json.as_bytes());
    CursorToken { token }
  }
}
impl TryFrom<CursorToken> for cala_ledger::entry::EntriesByCreatedAtCursor {
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

fn layer_to_string(layer: cala_ledger::Layer) -> String {
  match layer {
    cala_ledger::Layer::Settled => "Settled".to_string(),
    cala_ledger::Layer::Pending => "Pending".to_string(),
    cala_ledger::Layer::Encumbrance => "Encumbrance".to_string(),
  }
}

impl From<cala_ledger::entry::Entry> for EntryValues {
  fn from(entry: cala_ledger::entry::Entry) -> Self {
    let values = entry.into_values();
    Self {
      id: values.id.to_string(),
      version: values.version,
      transaction_id: values.transaction_id.to_string(),
      journal_id: values.journal_id.to_string(),
      account_id: values.account_id.to_string(),
      entry_type: values.entry_type.to_string(),
      sequence: values.sequence,
      layer: layer_to_string(values.layer),
      units: values.units.to_string(),
      currency: values.currency.to_string(),
      direction: values.direction.to_string(),
      description: values.description,
      metadata: values.metadata,
    }
  }
}
