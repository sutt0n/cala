import {
  CalaLedger,
  NewTxTemplateTransactionValues,
  NewTxTemplateEntryValues,
  NewParamDefinitionValues,
  ParamDataTypeValues,
  CalaTransaction,
} from "@galoymoney/cala-ledger";

const main = async () => {
  const pgHost = process.env.PG_HOST || "localhost";
  const pgCon = `postgres://user:password@${pgHost}:5433/pg`;

  const cala = await CalaLedger.connect({
    pgCon,
    outbox: { enabled: true, listenPort: 2258 },
  });
  console.log("CalaLedger connected");

  const account = await cala.accounts().create({
    name: "MY NAME",
    code: "USERS_ONE",
    metadata: {
      something: "users",
      more: true,
    },
  });
  console.log("Account created", account.id());
  const account2 = await cala.accounts().create({
    name: "MY NAME",
    code: "USERS_TWO",
    metadata: {
      something: "users",
      more: true,
    },
  });

  console.log("Account created", account2.id());

  let result = await cala.accounts().list({ first: 1 });
  console.log("First accounts: ", result);

  result = await cala.accounts().list({ first: 1, after: result.endCursor });
  console.log("Next accounts: ", result);

  const journal = await cala.journals().create({
    name: "MY JOURNAL",
    description: "MY DESCRIPTION",
    code: "MY_JOURNAL",
  });

  console.log("Journal Created", journal.id());

  const recordDepositDrEntry: NewTxTemplateEntryValues = {
    entryType: "'RECORD_DEPOSIT_DR'",
    currency: "params.currency",
    accountId: "params.deposit_omnibus_account_id",
    direction: "DEBIT",
    layer: "SETTLED",
    units: "params.amount",
  };

  const recordDepositCrEntry: NewTxTemplateEntryValues = {
    entryType: "'RECORD_DEPOSIT_CR'",
    currency: "params.currency",
    accountId: "params.credit_account_id",
    direction: "CREDIT",
    layer: "SETTLED",
    units: "params.amount",
  };

  const txInput: NewTxTemplateTransactionValues = {
    journalId: "params.journal_id",
    effective: "params.effective",
    metadata: "params.meta",
    externalId: "params.external_id",
    description: "'Record a deposit'",
  };

  const txParams: NewParamDefinitionValues[] = [
    {
      name: "external_id",
      type: ParamDataTypeValues.String,
    },
    {
      name: "journal_id",
      type: ParamDataTypeValues.Uuid,
    },
    {
      name: "currency",
      type: ParamDataTypeValues.String,
    },
    {
      name: "amount",
      type: ParamDataTypeValues.Decimal,
    },
    {
      name: "deposit_omnibus_account_id",
      type: ParamDataTypeValues.Uuid,
    },
    {
      name: "credit_account_id",
      type: ParamDataTypeValues.Uuid,
    },
    {
      name: "effective",
      type: ParamDataTypeValues.Date,
    },
    {
      name: "meta",
      type: ParamDataTypeValues.Json,
    },
  ];

  const txTemplate = await cala.txTemplates().create({
    code: "RECORD_DEPOSIT",
    description: "Record deposit transaction",
    externalId: "RECORD_DEPOSIT_v0.1",
    entries: [recordDepositDrEntry, recordDepositCrEntry],
    transaction: txInput,
    params: txParams,
  });

  console.log(
    "Tx Template Created",
    txTemplate.values().id,
    txTemplate.values().code,
  );

  const txTemplates = await cala.txTemplates().list({
    first: 10,
  });

  console.log("Tx Templates: ", txTemplates);

  const retrievedTxTemplate = await cala
    .txTemplates()
    .findByCode("RECORD_DEPOSIT");

  console.log("Retrieved Tx Template", retrievedTxTemplate.values());

  const transactionParams = {
    journal_id: journal.id(),
    external_id: "transaction_external_id-123",
    currency: "USD",
    amount: 100.0,
    deposit_omnibus_account_id: account.id(),
    credit_account_id: account2.id(),
    effective: new Date().toISOString(),
    meta: { something: "useful" },
  };

  const tx: CalaTransaction = await cala
    .transactions()
    .post(txTemplate.values().code, transactionParams);

  const balances1 = await cala
    .balances()
    .find(account.id(), journal.id(), "USD");
  const balances2 = await cala
    .balances()
    .find(account2.id(), journal.id(), "USD");

  console.log("Balances for account 1:", balances1);
  console.log("Balances for account 2:", balances2);

  const entries1 = await cala
    .entries()
    .listForAccountId(account.id(), { first: 10 });
  const entries2 = await cala
    .entries()
    .listForAccountId(account2.id(), { first: 10 });

  console.log("Entries for account 1:", entries1);
  console.log("Entries for account 2:", entries2);

  const txEntries = await cala.entries().listByTransaction(tx.id());

  console.log("Entries for transaction:", txEntries);
};

main();
