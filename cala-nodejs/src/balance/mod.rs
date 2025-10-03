use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[napi]
pub struct CalaBalances {
  inner: cala_ledger::balance::Balances,
}

#[napi(object)]
pub struct BalanceValues {
  pub pending: String,
  pub settled: String,
  pub encumbrance: String,
}

#[napi]
impl CalaBalances {
  pub fn new(inner: &cala_ledger::balance::Balances) -> Self {
    Self {
      inner: inner.clone(),
    }
  }

  #[napi]
  pub async fn find(
    &self,
    account_id: String,
    journal_id: String,
    currency: String,
  ) -> napi::Result<BalanceValues> {
    let account_id = account_id
      .parse::<cala_ledger::AccountId>()
      .map_err(crate::generic_napi_error)?;
    let journal_id = journal_id
      .parse::<cala_ledger::JournalId>()
      .map_err(crate::generic_napi_error)?;
    let currency = currency
      .parse::<cala_ledger::Currency>()
      .map_err(crate::generic_napi_error)?;

    let balance = self
      .inner
      .find(journal_id, account_id, currency)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(BalanceValues {
      pending: balance.pending().to_string(),
      settled: balance.settled().to_string(),
      encumbrance: balance.encumbrance().to_string(),
    })
  }

  pub async fn find_by_account(&self, account_id: String) -> napi::Result<Vec<BalanceValues>> {
    let account_id = account_id
      .parse::<cala_ledger::AccountId>()
      .map_err(crate::generic_napi_error)?;

    let balances = self
      .inner
      .find_all_for_account(account_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(
      balances
        .into_iter()
        .map(|balance| BalanceValues {
          pending: balance.pending().to_string(),
          settled: balance.settled().to_string(),
          encumbrance: balance.encumbrance().to_string(),
        })
        .collect(),
    )
  }
}
