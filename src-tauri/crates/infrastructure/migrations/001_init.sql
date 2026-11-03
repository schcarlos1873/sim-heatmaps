-- Convenções: tabelas no plural em snake_case, PK `id`, FKs `<entidade>_id`,
-- booleanos INTEGER 0/1, datas TEXT ISO-8601 (`created_at`, `updated_at`).

CREATE TABLE machines (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    scanner_model   TEXT    NOT NULL DEFAULT 'NP800',
    width_mm        REAL    NOT NULL,
    bin_count       INTEGER NOT NULL DEFAULT 800,
    speed_m_min     REAL    NOT NULL,
    scan_time_s     REAL    NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);

CREATE TABLE properties (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_id    INTEGER NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
    code          TEXT    NOT NULL COLLATE NOCASE,
    name          TEXT    NOT NULL,
    unit          TEXT    NOT NULL DEFAULT '',
    decimals      INTEGER NOT NULL DEFAULT 2,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    active        INTEGER NOT NULL DEFAULT 1,
    target        REAL    NOT NULL,
    error_high    REAL    NOT NULL,
    warning_high  REAL    NOT NULL,
    warning_low   REAL    NOT NULL,
    error_low     REAL    NOT NULL,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    UNIQUE (machine_id, code)
);

CREATE INDEX ix_properties_machine_id ON properties (machine_id, sort_order);

-- Índice das bobinas geradas. Os valores medidos ficam somente nos arquivos CSV.
CREATE TABLE reels (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    reel_id       TEXT    NOT NULL,
    machine_id    INTEGER REFERENCES machines(id) ON DELETE SET NULL,
    machine_name  TEXT    NOT NULL,
    started_at    TEXT    NOT NULL,
    ended_at      TEXT    NOT NULL,
    scan_count    INTEGER NOT NULL,
    bin_count     INTEGER NOT NULL,
    directory     TEXT    NOT NULL,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')),
    UNIQUE (machine_name, reel_id)
);

CREATE INDEX ix_reels_machine_id ON reels (machine_id, started_at);

CREATE TABLE reel_files (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    reel_id        INTEGER NOT NULL REFERENCES reels(id) ON DELETE CASCADE,
    property_code  TEXT    NOT NULL,
    property_name  TEXT    NOT NULL,
    file_path      TEXT    NOT NULL,
    sort_order     INTEGER NOT NULL
);

CREATE INDEX ix_reel_files_reel_id ON reel_files (reel_id, sort_order);

CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'))
);
