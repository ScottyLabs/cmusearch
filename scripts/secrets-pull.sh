#!/usr/bin/env bash
export VAULT_ADDR=https://secrets.scottylabs.org

vault kv get -format=json ScottyLabs/cmusearch |
    jq -r '.data.data | to_entries[] | "\(.key)=\"\(.value)\""' >data/.env
echo "Pulled from vault: ScottyLabs/cmusearch"
