mod values;

use cala_ledger::{velocity::Params, TransactionId};
use cel_interpreter::CelValue;
use chrono::DateTime;
use rust_decimal::Decimal;
use std::{collections::HashMap, sync::Arc};

use values::*;

#[napi]
pub struct CalaTransactions {
  inner: cala_ledger::transaction::Transactions,
  ledger: Arc<cala_ledger::CalaLedger>,
}

#[napi]
pub struct CalaTransaction {
  inner: cala_ledger::transaction::Transaction,
}

#[napi]
impl CalaTransaction {
  #[napi]
  pub fn id(&self) -> String {
    self.inner.id().to_string()
  }

  #[napi]
  pub fn values(&self) -> TransactionValues {
    TransactionValues::from(&self.inner)
  }
}
#[napi]
impl CalaTransactions {
  pub fn new(
    inner: &cala_ledger::transaction::Transactions,
    ledger: Arc<cala_ledger::CalaLedger>,
  ) -> Self {
    Self {
      inner: inner.clone(),
      ledger,
    }
  }

  #[napi]
  pub async fn find_by_id(&self, id: String) -> napi::Result<CalaTransaction> {
    let tx_id = id
      .parse::<cala_ledger::TransactionId>()
      .map_err(crate::generic_napi_error)?;

    match self.inner.find_by_id(tx_id).await {
      Ok(tx) => Ok(CalaTransaction { inner: tx }),
      Err(cala_ledger::transaction::error::TransactionError::CouldNotFindById(_)) => Err(
        napi::Error::from_reason(format!("Transaction with id {} not found", id)),
      ),
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }

  #[napi]
  pub async fn find_by_external_id(
    &self,
    external_id: String,
  ) -> napi::Result<Option<CalaTransaction>> {
    match self.inner.find_by_external_id(external_id.clone()).await {
      Ok(tx) => Ok(Some(CalaTransaction { inner: tx })),
      Err(cala_ledger::transaction::error::TransactionError::CouldNotFindByExternalId(_)) => {
        Err(napi::Error::from_reason(format!(
          "Could not find transaction with external_id {}",
          external_id
        )))
      }
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }

  #[napi]
  pub async fn post(
    &self,
    tx_template_code: String,
    params: serde_json::Value,
  ) -> napi::Result<CalaTransaction> {
    let id = TransactionId::new();

    let template = self
      .ledger
      .tx_templates()
      .find_by_code(&tx_template_code)
      .await
      .map_err(crate::generic_napi_error)?;

    let param_definitions = template.values().params.clone();

    let mut param_types: HashMap<String, cala_types::param::ParamDataType> = HashMap::new();
    if let Some(params) = param_definitions {
      for param_def in params {
        param_types.insert(param_def.name.clone(), param_def.r#type);
      }
    }

    let hashmap_params: HashMap<String, serde_json::Value> = serde_json::from_value(params.clone())
      .map_err(|e| napi::Error::from_reason(format!("Invalid params: {}", e)))?;

    let mut params = Params::new();

    // iterate over the hashmap and insert each key-value pair into params
    for (key, value) in hashmap_params {
      // Check if this parameter is defined as Decimal in the template
      let cel_value = if let Some(param_type) = param_types.get(&key) {
        if *param_type == cala_types::param::ParamDataType::Decimal {
          match &value {
            serde_json::Value::Number(n) => {
              if let Ok(decimal) = n.to_string().parse::<Decimal>() {
                CelValue::from(decimal)
              } else {
                CelValue::from(value)
              }
            }
            _ => CelValue::from(value),
          }
        } else if *param_type == cala_types::param::ParamDataType::Date {
          let parsed_naive_date = match value {
            serde_json::Value::String(ref s) => DateTime::parse_from_rfc3339(s)
              .map(|dt| dt.naive_utc().date())
              .map_err(|e| {
                napi::Error::from_reason(format!("Invalid date format for key '{}': {}", key, e))
              })?,
            _ => {
              return Err(napi::Error::from_reason(format!(
                "Expected string for date parameter '{}'",
                key
              )))
            }
          };
          CelValue::from(parsed_naive_date)
        } else {
          CelValue::from(value)
        }
      } else {
        CelValue::from(value)
      };
      params.insert(key, cel_value);
    }

    let transaction = self
      .ledger
      .post_transaction(id, &tx_template_code, params)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }

  #[napi]
  pub async fn void_transaction(&self, transaction_id: String) -> napi::Result<CalaTransaction> {
    let void_tx_id = TransactionId::new();

    let existing_tx_id = transaction_id
      .parse::<cala_ledger::TransactionId>()
      .map_err(crate::generic_napi_error)?;

    let transaction = self
      .ledger
      .void_transaction(void_tx_id, existing_tx_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }
}
