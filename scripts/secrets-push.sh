#!/usr/bin/env bash
export VAULT_ADDR=https://secrets.scottylabs.org

cat data/.env | xargs -r vault kv put -mount="ScottyLabs" "cmusearch"
