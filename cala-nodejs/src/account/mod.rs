mod values;

use values::*;

use super::query::*;

#[napi(object)]
pub struct NewAccount {
  pub id: Option<String>,
  pub code: String,
  pub name: String,
  pub external_id: Option<String>,
  pub description: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
pub struct PaginatedAccounts {
  pub accounts: Vec<AccountValues>,
  pub has_next_page: bool,
  pub end_cursor: Option<CursorToken>,
}

#[napi]
pub struct CalaAccounts {
  inner: cala_ledger::account::Accounts,
}

#[napi]
pub struct CalaAccount {
  inner: cala_ledger::account::Account,
}

#[napi]
impl CalaAccount {
  #[napi]
  pub fn id(&self) -> String {
    self.inner.id().to_string()
  }

  #[napi]
  pub fn values(&self) -> AccountValues {
    AccountValues::from(&self.inner)
  }
}

#[napi]
impl CalaAccounts {
  pub fn new(inner: &cala_ledger::account::Accounts) -> Self {
    Self {
      inner: inner.clone(),
    }
  }

  #[napi]
  pub async fn create(&self, new_account: NewAccount) -> napi::Result<CalaAccount> {
    let id = if let Some(id) = new_account.id {
      id.parse::<cala_ledger::AccountId>()
        .map_err(crate::generic_napi_error)?
    } else {
      cala_ledger::AccountId::new()
    };

    let mut new = cala_ledger::account::NewAccount::builder();
    new.id(id).code(new_account.code).name(new_account.name);

    if let Some(external_id) = new_account.external_id {
      new.external_id(external_id);
    }

    if let Some(description) = new_account.description {
      new.description(description);
    }

    if let Some(metadata) = new_account.metadata {
      new.metadata(metadata).map_err(crate::generic_napi_error)?;
    }

    let account = self
      .inner
      .create(new.build().map_err(crate::generic_napi_error)?)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaAccount { inner: account })
  }

  #[napi]
  pub async fn find_by_code(&self, code: String) -> napi::Result<CalaAccount> {
    match self.inner.find_by_code(code.clone()).await {
      Ok(account) => Ok(CalaAccount { inner: account }),
      Err(cala_ledger::account::error::AccountError::CouldNotFindByCode(_)) => Err(
        napi::Error::from_reason(format!("Account with code '{}' not found", code)),
      ),
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }

  #[napi]
  pub async fn find_by_id(&self, account_id: String) -> napi::Result<CalaAccount> {
    let account_id = account_id
      .parse::<cala_ledger::AccountId>()
      .map_err(crate::generic_napi_error)?;

    match self.inner.find(account_id).await {
      Ok(account) => Ok(CalaAccount { inner: account }),
      Err(cala_ledger::account::error::AccountError::CouldNotFindById(_)) => Err(
        napi::Error::from_reason(format!("Account with id '{}' not found", account_id)),
      ),
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }

  #[napi]
  pub async fn find_by_external_id(&self, external_id: String) -> napi::Result<CalaAccount> {
    match self.inner.find_by_external_id(external_id).await {
      Ok(account) => Ok(CalaAccount { inner: account }),
      Err(cala_ledger::account::error::AccountError::CouldNotFindByExternalId(external_id)) => {
        Err(napi::Error::from_reason(format!(
          "Account with external_id '{}' not found",
          external_id
        )))
      }
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }

  #[napi]
  pub async fn list(&self, query: PaginatedQueryArgs) -> napi::Result<PaginatedAccounts> {
    let query = cala_ledger::es_entity::PaginatedQueryArgs {
      after: query.after.map(|c| c.try_into()).transpose()?,
      first: usize::try_from(query.first).map_err(crate::generic_napi_error)?,
    };

    let ret = match self.inner.list(query).await {
      Ok(ret) => ret,
      Err(e) => return Err(crate::generic_napi_error(e)),
    };

    Ok(PaginatedAccounts {
      accounts: ret.entities.into_iter().map(AccountValues::from).collect(),
      has_next_page: ret.has_next_page,
      end_cursor: ret.end_cursor.map(|c| c.into()),
    })
  }
}
