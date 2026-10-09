ALTER TABLE contract_events ADD COLUMN event_index INTEGER;

WITH indexed_events AS (
    SELECT id, ROW_NUMBER() OVER (
        PARTITION BY tx_hash
        ORDER BY id
    ) - 1 AS event_index
    FROM contract_events
)
UPDATE contract_events
SET event_index = indexed_events.event_index
FROM indexed_events
WHERE contract_events.id = indexed_events.id;

ALTER TABLE contract_events ALTER COLUMN event_index SET NOT NULL;
CREATE UNIQUE INDEX idx_events_tx_event ON contract_events (tx_hash, event_index);

CREATE TABLE ledger_checkpoints (
    sequence INTEGER PRIMARY KEY,
    closed_at BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO ledger_checkpoints (sequence, closed_at)
SELECT MAX(ledger_sequence), 0
FROM transactions
HAVING COUNT(*) > 0
ON CONFLICT (sequence) DO NOTHING;
