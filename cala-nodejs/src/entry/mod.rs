mod values;

use cala_ledger::es_entity::ListDirection;
use values::*;

use crate::query::{self, PaginatedQueryArgs};

#[napi]
pub struct CalaEntries {
  inner: cala_ledger::entry::Entries,
}

#[napi]
impl CalaEntries {
  pub fn new(inner: &cala_ledger::entry::Entries) -> Self {
    Self {
      inner: inner.clone(),
    }
  }

  #[napi]
  pub async fn list_by_transaction(
    &self,
    transaction_id: String,
  ) -> napi::Result<Vec<EntryValues>> {
    let transaction_id = transaction_id
      .parse::<cala_ledger::TransactionId>()
      .map_err(crate::generic_napi_error)?;

    let entries = self
      .inner
      .list_for_transaction_id(transaction_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(entries.into_iter().map(EntryValues::from).collect())
  }

  #[napi]
  pub async fn list_for_journal_id(
    &self,
    journal_id: String,
    query: PaginatedQueryArgs,
    direction: Option<query::ListDirection>,
  ) -> napi::Result<PaginatedEntries> {
    let query = cala_ledger::es_entity::PaginatedQueryArgs {
      after: query.after.map(|c| c.try_into()).transpose()?,
      first: usize::try_from(query.first).map_err(crate::generic_napi_error)?,
    };

    let journal_id = journal_id
      .parse::<cala_ledger::JournalId>()
      .map_err(crate::generic_napi_error)?;

    let list_direction = match direction {
      Some(query::ListDirection::Ascending) => ListDirection::Ascending,
      Some(query::ListDirection::Descending) => ListDirection::Descending,
      None => ListDirection::default(),
    };

    let ret = self
      .inner
      .list_for_journal_id(journal_id, query, list_direction)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(PaginatedEntries {
      entries: ret.entities.into_iter().map(EntryValues::from).collect(),
      has_next_page: ret.has_next_page,
      end_cursor: ret.end_cursor.map(|c| c.into()),
    })
  }

  #[napi]
  pub async fn list_for_account_id(
    &self,
    account_id: String,
    query: PaginatedQueryArgs,
    direction: Option<query::ListDirection>,
  ) -> napi::Result<PaginatedEntries> {
    let query = cala_ledger::es_entity::PaginatedQueryArgs {
      after: query.after.map(|c| c.try_into()).transpose()?,
      first: usize::try_from(query.first).map_err(crate::generic_napi_error)?,
    };

    let account_id = account_id
      .parse::<cala_ledger::AccountId>()
      .map_err(crate::generic_napi_error)?;

    let list_direction = match direction {
      Some(query::ListDirection::Ascending) => ListDirection::Ascending,
      Some(query::ListDirection::Descending) => ListDirection::Descending,
      None => ListDirection::default(),
    };

    let ret = self
      .inner
      .list_for_account_id(account_id, query, list_direction)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(PaginatedEntries {
      entries: ret.entities.into_iter().map(EntryValues::from).collect(),
      has_next_page: ret.has_next_page,
      end_cursor: ret.end_cursor.map(|c| c.into()),
    })
  }
}
