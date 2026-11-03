use chrono::NaiveDateTime;
use domain::machine::{Machine, ScannerModel};
use domain::property::{Limits, Property};
use domain::repositories::{
    MachineRepository, PropertyRepository, ReelFile, ReelRecord, ReelRepository, SettingsRepository,
};
use domain::{DomainError, DomainResult};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::{storage, Database};

const DT: &str = "%Y-%m-%dT%H:%M:%S";

fn parse_dt(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, DT).unwrap_or_default()
}

fn not_found(changed: usize, entity: &'static str) -> DomainResult<()> {
    if changed == 0 { Err(DomainError::NotFound { entity }) } else { Ok(()) }
}

// ---------------------------------------------------------------- machines

pub struct SqliteMachineRepository(pub Database);

const MACHINE_COLS: &str = "id, name, scanner_model, width_mm, bin_count, speed_m_min, scan_time_s";

fn map_machine(r: &Row) -> rusqlite::Result<Machine> {
    let model: String = r.get(2)?;
    Ok(Machine {
        id: Some(r.get(0)?),
        name: r.get(1)?,
        scanner_model: ScannerModel::parse(&model)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(e)))?,
        width_mm: r.get(3)?,
        bin_count: r.get(4)?,
        speed_m_min: r.get(5)?,
        scan_time_s: r.get(6)?,
    })
}

impl MachineRepository for SqliteMachineRepository {
    fn list(&self) -> DomainResult<Vec<Machine>> {
        let conn = self.0.conn();
        let mut stmt = conn.prepare(&format!("SELECT {MACHINE_COLS} FROM machines ORDER BY name")).map_err(storage)?;
        let rows = stmt.query_map([], map_machine).map_err(storage)?;
        rows.collect::<Result<_, _>>().map_err(storage)
    }

    fn get(&self, id: i64) -> DomainResult<Option<Machine>> {
        self.0
            .conn()
            .query_row(&format!("SELECT {MACHINE_COLS} FROM machines WHERE id = ?1"), [id], map_machine)
            .optional()
            .map_err(storage)
    }

    fn find_by_name(&self, name: &str) -> DomainResult<Option<Machine>> {
        self.0
            .conn()
            .query_row(&format!("SELECT {MACHINE_COLS} FROM machines WHERE name = ?1"), [name], map_machine)
            .optional()
            .map_err(storage)
    }

    fn insert(&self, m: &Machine) -> DomainResult<i64> {
        let conn = self.0.conn();
        conn.execute(
            "INSERT INTO machines (name, scanner_model, width_mm, bin_count, speed_m_min, scan_time_s)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![m.name, m.scanner_model.as_str(), m.width_mm, m.bin_count, m.speed_m_min, m.scan_time_s],
        )
        .map_err(storage)?;
        Ok(conn.last_insert_rowid())
    }

    fn update(&self, m: &Machine) -> DomainResult<()> {
        let changed = self
            .0
            .conn()
            .execute(
                "UPDATE machines SET name = ?2, scanner_model = ?3, width_mm = ?4, bin_count = ?5, speed_m_min = ?6,
                 scan_time_s = ?7, updated_at = strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime') WHERE id = ?1",
                params![m.id, m.name, m.scanner_model.as_str(), m.width_mm, m.bin_count, m.speed_m_min, m.scan_time_s],
            )
            .map_err(storage)?;
        not_found(changed, "machine")
    }

    fn delete(&self, id: i64) -> DomainResult<()> {
        let changed = self.0.conn().execute("DELETE FROM machines WHERE id = ?1", [id]).map_err(storage)?;
        not_found(changed, "machine")
    }

    fn count(&self) -> DomainResult<i64> {
        self.0.conn().query_row("SELECT COUNT(*) FROM machines", [], |r| r.get(0)).map_err(storage)
    }
}

// -------------------------------------------------------------- properties

pub struct SqlitePropertyRepository(pub Database);

const PROPERTY_COLS: &str = "id, machine_id, code, name, unit, decimals, sort_order, active,
    target, error_high, warning_high, warning_low, error_low";

fn map_property(r: &Row) -> rusqlite::Result<Property> {
    Ok(Property {
        id: Some(r.get(0)?),
        machine_id: r.get(1)?,
        code: r.get(2)?,
        name: r.get(3)?,
        unit: r.get(4)?,
        decimals: r.get(5)?,
        sort_order: r.get(6)?,
        active: r.get(7)?,
        limits: Limits {
            target: r.get(8)?,
            error_high: r.get(9)?,
            warning_high: r.get(10)?,
            warning_low: r.get(11)?,
            error_low: r.get(12)?,
        },
    })
}

impl PropertyRepository for SqlitePropertyRepository {
    fn list_by_machine(&self, machine_id: i64) -> DomainResult<Vec<Property>> {
        let conn = self.0.conn();
        let mut stmt = conn
            .prepare(&format!("SELECT {PROPERTY_COLS} FROM properties WHERE machine_id = ?1 ORDER BY sort_order, id"))
            .map_err(storage)?;
        let rows = stmt.query_map([machine_id], map_property).map_err(storage)?;
        rows.collect::<Result<_, _>>().map_err(storage)
    }

    fn get(&self, id: i64) -> DomainResult<Option<Property>> {
        self.0
            .conn()
            .query_row(&format!("SELECT {PROPERTY_COLS} FROM properties WHERE id = ?1"), [id], map_property)
            .optional()
            .map_err(storage)
    }

    fn find_by_code(&self, machine_id: i64, code: &str) -> DomainResult<Option<Property>> {
        self.0
            .conn()
            .query_row(
                &format!("SELECT {PROPERTY_COLS} FROM properties WHERE machine_id = ?1 AND code = ?2"),
                params![machine_id, code],
                map_property,
            )
            .optional()
            .map_err(storage)
    }

    fn insert(&self, p: &Property) -> DomainResult<i64> {
        let conn = self.0.conn();
        let l = &p.limits;
        conn.execute(
            "INSERT INTO properties (machine_id, code, name, unit, decimals, sort_order, active,
             target, error_high, warning_high, warning_low, error_low)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                p.machine_id, p.code, p.name, p.unit, p.decimals, p.sort_order, p.active,
                l.target, l.error_high, l.warning_high, l.warning_low, l.error_low
            ],
        )
        .map_err(storage)?;
        Ok(conn.last_insert_rowid())
    }

    fn update(&self, p: &Property) -> DomainResult<()> {
        let l = &p.limits;
        let changed = self
            .0
            .conn()
            .execute(
                "UPDATE properties SET machine_id = ?2, code = ?3, name = ?4, unit = ?5, decimals = ?6, sort_order = ?7,
                 active = ?8, target = ?9, error_high = ?10, warning_high = ?11, warning_low = ?12, error_low = ?13,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')
                 WHERE id = ?1",
                params![
                    p.id, p.machine_id, p.code, p.name, p.unit, p.decimals, p.sort_order, p.active,
                    l.target, l.error_high, l.warning_high, l.warning_low, l.error_low
                ],
            )
            .map_err(storage)?;
        not_found(changed, "property")
    }

    fn delete(&self, id: i64) -> DomainResult<()> {
        let changed = self.0.conn().execute("DELETE FROM properties WHERE id = ?1", [id]).map_err(storage)?;
        not_found(changed, "property")
    }
}

// ------------------------------------------------------------------- reels

pub struct SqliteReelRepository(pub Database);

const REEL_COLS: &str =
    "id, reel_id, machine_id, machine_name, started_at, ended_at, scan_count, bin_count, directory, created_at";

fn map_reel(r: &Row) -> rusqlite::Result<ReelRecord> {
    Ok(ReelRecord {
        id: Some(r.get(0)?),
        reel_id: r.get(1)?,
        machine_id: r.get(2)?,
        machine_name: r.get(3)?,
        started_at: parse_dt(&r.get::<_, String>(4)?),
        ended_at: parse_dt(&r.get::<_, String>(5)?),
        scan_count: r.get(6)?,
        bin_count: r.get(7)?,
        directory: r.get(8)?,
        files: vec![],
        created_at: Some(parse_dt(&r.get::<_, String>(9)?)),
    })
}

impl SqliteReelRepository {
    fn load_files(conn: &Connection, reel: &mut ReelRecord) -> rusqlite::Result<()> {
        let mut stmt = conn.prepare(
            "SELECT property_code, property_name, file_path FROM reel_files WHERE reel_id = ?1 ORDER BY sort_order, id",
        )?;
        reel.files = stmt
            .query_map([reel.id], |r| Ok(ReelFile { property_code: r.get(0)?, property_name: r.get(1)?, file_path: r.get(2)? }))?
            .collect::<Result<_, _>>()?;
        Ok(())
    }

    fn query(&self, sql: &str, params: impl rusqlite::Params) -> DomainResult<Vec<ReelRecord>> {
        let conn = self.0.conn();
        let mut stmt = conn.prepare(sql).map_err(storage)?;
        let mut reels: Vec<ReelRecord> =
            stmt.query_map(params, map_reel).map_err(storage)?.collect::<Result<_, _>>().map_err(storage)?;
        for r in &mut reels {
            Self::load_files(&conn, r).map_err(storage)?;
        }
        Ok(reels)
    }
}

impl ReelRepository for SqliteReelRepository {
    fn list(&self, machine_id: Option<i64>, limit: u32) -> DomainResult<Vec<ReelRecord>> {
        self.query(
            &format!(
                "SELECT {REEL_COLS} FROM reels WHERE ?1 IS NULL OR machine_id = ?1
                 ORDER BY started_at DESC, id DESC LIMIT ?2"
            ),
            params![machine_id, limit],
        )
    }

    fn get(&self, id: i64) -> DomainResult<Option<ReelRecord>> {
        Ok(self.query(&format!("SELECT {REEL_COLS} FROM reels WHERE id = ?1"), [id])?.into_iter().next())
    }

    fn find(&self, machine_name: &str, reel_id: &str) -> DomainResult<Option<ReelRecord>> {
        Ok(self
            .query(&format!("SELECT {REEL_COLS} FROM reels WHERE machine_name = ?1 AND reel_id = ?2"), [machine_name, reel_id])?
            .into_iter()
            .next())
    }

    fn upsert(&self, r: &ReelRecord) -> DomainResult<i64> {
        let mut conn = self.0.conn();
        let tx = conn.transaction().map_err(storage)?;
        tx.execute("DELETE FROM reels WHERE machine_name = ?1 AND reel_id = ?2", params![r.machine_name, r.reel_id])
            .map_err(storage)?;
        tx.execute(
            "INSERT INTO reels (reel_id, machine_id, machine_name, started_at, ended_at, scan_count, bin_count, directory)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                r.reel_id,
                r.machine_id,
                r.machine_name,
                r.started_at.format(DT).to_string(),
                r.ended_at.format(DT).to_string(),
                r.scan_count,
                r.bin_count,
                r.directory
            ],
        )
        .map_err(storage)?;
        let pk = tx.last_insert_rowid();
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO reel_files (reel_id, property_code, property_name, file_path, sort_order)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(storage)?;
            for (i, f) in r.files.iter().enumerate() {
                stmt.execute(params![pk, f.property_code, f.property_name, f.file_path, i as i64]).map_err(storage)?;
            }
        }
        tx.commit().map_err(storage)?;
        Ok(pk)
    }

    fn delete(&self, id: i64) -> DomainResult<()> {
        let changed = self.0.conn().execute("DELETE FROM reels WHERE id = ?1", [id]).map_err(storage)?;
        not_found(changed, "reel")
    }
}

// ---------------------------------------------------------------- settings

pub struct SqliteSettingsRepository(pub Database);

impl SettingsRepository for SqliteSettingsRepository {
    fn get(&self, key: &str) -> DomainResult<Option<String>> {
        self.0
            .conn()
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
            .optional()
            .map_err(storage)
    }

    fn set(&self, key: &str, value: &str) -> DomainResult<()> {
        self.0
            .conn()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime')",
                [key, value],
            )
            .map_err(storage)?;
        Ok(())
    }
}
