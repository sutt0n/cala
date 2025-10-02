mod helpers;

use cala_ledger::{CalaLedger, CalaLedgerConfig};
use rand::distr::{Alphanumeric, SampleString};

#[tokio::test]
async fn journal_find_by_code() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool)
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let code = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let new_journal = helpers::test_journal_with_code(&code);
    let journal = cala.journals().create(new_journal).await.unwrap();

    let journal_by_code = cala
        .journals()
        .find_by_code(code.to_string())
        .await
        .unwrap();

    assert_eq!(journal.id(), journal_by_code.id());

    Ok(())
}

#[tokio::test]
async fn journal_cannot_find_by_code() -> anyhow::Result<()> {
    let pool = helpers::init_pool().await?;
    let cala_config = CalaLedgerConfig::builder()
        .pool(pool)
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;

    let code = Alphanumeric.sample_string(&mut rand::rng(), 16);
    let result = cala.journals().find_by_code(code.to_string()).await;

    assert!(result.is_err());

    Ok(())
}
