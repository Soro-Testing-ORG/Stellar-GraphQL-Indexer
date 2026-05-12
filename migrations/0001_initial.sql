-- Initial schema for the Stellar GraphQL Indexer

CREATE TABLE IF NOT EXISTS transactions (
    hash             TEXT        PRIMARY KEY,
    ledger_sequence  INTEGER     NOT NULL,
    source_account   TEXT        NOT NULL,
    fee              INTEGER     NOT NULL,
    successful       BOOLEAN     NOT NULL,
    created_at       BIGINT      NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_ledger ON transactions (ledger_sequence);
CREATE INDEX IF NOT EXISTS idx_transactions_source ON transactions (source_account);

CREATE TABLE IF NOT EXISTS contract_events (
    id               BIGSERIAL   PRIMARY KEY,
    contract_id      TEXT        NOT NULL,
    ledger_sequence  INTEGER     NOT NULL,
    tx_hash          TEXT        NOT NULL REFERENCES transactions(hash),
    topics           JSONB       NOT NULL DEFAULT '[]',
    data             JSONB       NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_events_contract ON contract_events (contract_id);
CREATE INDEX IF NOT EXISTS idx_events_ledger   ON contract_events (ledger_sequence);
