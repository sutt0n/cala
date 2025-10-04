#!/usr/bin/env bats

load "helpers"

setup_file() {
  reset_pg_and_restart_server
}

teardown_file() {
  stop_server
  stop_nodejs_example
}

reset_pg_and_restart_server() {
  stop_server
  reset_pg
  PG_CON=$PG_CON_EXAMPLE start_server
}

@test "nodejs: entities sync to server" {
  reset_pg_and_restart_server

  exec_graphql 'list-accounts'
  accounts_before=$(graphql_output '.data.accounts.nodes | length')

  job_id=$(random_uuid)
  variables=$(
    jq -n \
      --arg jobId "$job_id" \
    '{
      input: {
        jobId: $jobId,
        endpoint: "http://localhost:2258"
      }
    }'
  )
  exec_graphql 'cala-outbox-import-job-create' "$variables"
  echo "GraphQL Response: $(graphql_output)"
  id=$(graphql_output '.data.calaOutboxImportJobCreate.job.jobId')
  error_msg=$(graphql_output '.errors[0].message')
  [[ "$id" == "$job_id" || "$error_msg" =~ duplicate.*jobs_name_key ]] || exit 1;

  background bash -c "cd ${REPO_ROOT}/examples/nodejs && npm run start > ${REPO_ROOT}/.nodejs-example-logs 2>&1" &
  NODEJS_EXAMPLE_PID=$!
  echo $NODEJS_EXAMPLE_PID > "${NODEJS_EXAMPLE_PID_FILE}"

  job_count=$(cat .e2e-logs | grep 'Executing CalaOutboxImportJob importing' | wc -l)
  retry 30 1 wait_for_new_import_job $job_count || true
  sleep 1

  for i in {1..90}; do
    exec_graphql 'list-accounts'
    accounts_after=$(graphql_output '.data.accounts.nodes | length')
    if [[ "$accounts_after" -gt "$accounts_before" ]] then
      break;
    fi
    sleep 1
  done

  # accounts
  [[ "$accounts_after" -gt "$accounts_before" ]] || exit 1

  # tx template
  variables=$(
    jq -n \
      --arg code "RECORD_DEPOSIT" \
    '{"code": $code}'
  )
  exec_graphql 'tx-template-find-by-code' "$variables"
  tx_template_code=$(graphql_output '.data.txTemplateFindByCode.txTemplate.code')
  [[ "$tx_template_code" != "RECORD_DEPOSIT" ]] || exit 1

  # journal by code
  variables=$(
    jq -n \
      --arg code "MY_JOURNAL" \
    '{"code": $code}'
  )
  exec_graphql 'journal-by-code' "$variables"
  journal_id=$(graphql_output '.data.journalByCode.journalId')
  [[ -n "$journal_id" ]] || exit 1

  # balance 
  variables=$(
    jq -n \
      --arg code "USERS_TWO" \
      --arg journalId "$journal_id" \
      --arg currency "USD" \
    '{"code": $code, "journalId": $journalId, "currency": $currency}'
  )
  exec_graphql 'account-by-code-with-balance' "$variables"
  balance=$(graphql_output '.data.accountByCode.balance.settled.normalBalance.units')
  echo "Balance: $balance"
  [[ "$balance" == "100" ]] || exit 1

  # transaction by external id
  variables=$(
    jq -n \
      --arg externalId "transaction_external_id-123" \
    '{"externalId": $externalId}'
  )
  exec_graphql 'transaction-by-external-id' "$variables"
  tx_external_id=$(graphql_output '.data.transactionByExternalId.externalId')
  [[ "$tx_external_id" == "transaction_external_id-123" ]] || exit 1

  # get account id
  variables=$(
    jq -n \
      --arg code "USERS_TWO" \
    '{"code": $code}'
  )
  exec_graphql 'account-by-code' "$variables"
  account_id=$(graphql_output '.data.accountByCode.accountId')

  echo "Account ID: $account_id"
  [[ -n "$account_id" ]] || exit 1

  # balances for account 
  variables=$(
    jq -n \
      --arg accountId "$account_id" \
    '{"accountId": $accountId}'
  )
  exec_graphql 'balances-for-account' "$variables"
  balance_count=$(graphql_output '.data.balancesForAccount | length')
  [[ "$balance_count" -gt 0 ]] || exit 1
}
