//! Leitura e escrita dos arquivos CSV do scanner (UTF-8 sem BOM, `;`, ponto decimal, CRLF).
//!
//! Um arquivo por bobina × propriedade, uma linha por scan (layout do "Show Numerical Values"):
//! ```text
//! #Format;sim-heatmaps-scan;1
//! #Reel;<reel_id>
//! #Machine;<nome>;<modelo>;<largura mm>;<bins>;<velocidade m/min>;<tempo de scan s>
//! #Property;<código>;<nome>;<unidade>;<casas decimais>
//! #Limits;<target>;<errorHigh>;<warningHigh>;<warningLow>;<errorLow>
//! Scan;StartTime;EndTime;Direction;Speed;MDPosition;EdgeFS;EdgeBS;SheetBreak;SensorStatus;SensorTemp;B1;…;Bn
//! 1;2026-11-02T08:00:00;2026-11-02T08:00:30;F;900.1;450;20.3;7979.8;0;0;35.2;;80.12;…
//! ```
//! Célula de bin vazia = sem dado (fora da borda, quebra de folha ou sensor inválido).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use application::ports::ReelStorage;
use application::{AppError, AppResult};
use chrono::NaiveDateTime;
use domain::format::format_number;
use domain::machine::{Machine, ScannerModel};
use domain::property::{Limits, Property};
use domain::scan::{Reel, Scan, ScanDirection, SensorStatus};

const EOL: &str = "\r\n";
const FORMAT: &str = "#Format;sim-heatmaps-scan;1";
const DT: &str = "%Y-%m-%dT%H:%M:%S%.f";
const FIXED_COLUMNS: [&str; 11] =
    ["Scan", "StartTime", "EndTime", "Direction", "Speed", "MDPosition", "EdgeFS", "EdgeBS", "SheetBreak", "SensorStatus", "SensorTemp"];
/// Casas decimais dos metadados (velocidade, posições, temperatura).
const META_DECIMALS: u8 = 3;

pub fn file_name(reel: &Reel) -> String {
    format!("{}_{}.csv", reel.reel_id, reel.property.code)
}

pub fn render(reel: &Reel) -> String {
    let m = &reel.machine;
    let p = &reel.property;
    let l = &p.limits;
    let num = |v: f64| format_number(v, META_DECIMALS);
    let mut out = String::with_capacity(reel.scans.len() * m.bin_count as usize * 7);
    let mut line = |s: String| {
        out.push_str(&s);
        out.push_str(EOL);
    };
    line(FORMAT.into());
    line(format!("#Reel;{}", reel.reel_id));
    line(format!(
        "#Machine;{};{};{};{};{};{}",
        m.name,
        m.scanner_model.as_str(),
        num(m.width_mm),
        m.bin_count,
        num(m.speed_m_min),
        num(m.scan_time_s)
    ));
    line(format!("#Property;{};{};{};{}", p.code, p.name, p.unit, p.decimals));
    line(format!(
        "#Limits;{};{};{};{};{}",
        num_full(l.target),
        num_full(l.error_high),
        num_full(l.warning_high),
        num_full(l.warning_low),
        num_full(l.error_low)
    ));
    let bins = (1..=m.bin_count).map(|i| format!("B{i}"));
    line(FIXED_COLUMNS.iter().map(|c| c.to_string()).chain(bins).collect::<Vec<_>>().join(";"));
    for s in &reel.scans {
        let mut row = vec![
            s.number.to_string(),
            s.started_at.format(DT).to_string(),
            s.ended_at.format(DT).to_string(),
            s.direction.code().into(),
            num(s.speed),
            num(s.md_position_m),
            num(s.edge_front_mm),
            num(s.edge_back_mm),
            u8::from(s.sheet_break).to_string(),
            s.sensor_status.code().to_string(),
            num(s.sensor_temp_c),
        ];
        row.extend(s.values.iter().map(|v| v.map(|v| format_number(v, p.decimals)).unwrap_or_default()));
        line(row.join(";"));
    }
    out
}

/// Limites com precisão total (não dependem das casas decimais da propriedade).
fn num_full(v: f64) -> String {
    format_number(v, 6)
}

fn parse_err(line: usize, msg: &str) -> AppError {
    AppError::new("parse", format!("line {}: {msg}", line + 1))
}

fn fields<'a>(lines: &[&'a str], index: usize, tag: &str, count: usize) -> AppResult<Vec<&'a str>> {
    let line = lines.get(index).ok_or_else(|| parse_err(index, &format!("missing {tag}")))?;
    let parts: Vec<&str> = line.split(';').collect();
    if parts.first() != Some(&tag) || parts.len() != count + 1 {
        return Err(parse_err(index, &format!("expected {tag} with {count} fields")));
    }
    Ok(parts[1..].to_vec())
}

fn number(text: &str, line: usize) -> AppResult<f64> {
    text.trim().parse::<f64>().ok().filter(|v| v.is_finite()).ok_or_else(|| parse_err(line, &format!("invalid number '{text}'")))
}

fn integer<T: std::str::FromStr>(text: &str, line: usize) -> AppResult<T> {
    text.trim().parse::<T>().map_err(|_| parse_err(line, &format!("invalid integer '{text}'")))
}

fn datetime(text: &str, line: usize) -> AppResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(text.trim(), DT).map_err(|_| parse_err(line, &format!("invalid date '{text}'")))
}

pub fn parse(text: &str) -> AppResult<Reel> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().map(|l| l.trim_end_matches('\r')).collect();
    if lines.first() != Some(&FORMAT) {
        return Err(parse_err(0, "not a sim-heatmaps scan file"));
    }
    let reel_id = fields(&lines, 1, "#Reel", 1)?[0].to_string();
    let f = fields(&lines, 2, "#Machine", 6)?;
    let machine = Machine {
        id: None,
        name: f[0].to_string(),
        scanner_model: ScannerModel::parse(f[1]).map_err(|_| parse_err(2, "invalid scanner model"))?,
        width_mm: number(f[2], 2)?,
        bin_count: integer(f[3], 2)?,
        speed_m_min: number(f[4], 2)?,
        scan_time_s: number(f[5], 2)?,
    };
    let f = fields(&lines, 3, "#Property", 4)?;
    let (code, name, unit, decimals) = (f[0].to_string(), f[1].to_string(), f[2].to_string(), integer(f[3], 3)?);
    let f = fields(&lines, 4, "#Limits", 5)?;
    let limits = Limits {
        target: number(f[0], 4)?,
        error_high: number(f[1], 4)?,
        warning_high: number(f[2], 4)?,
        warning_low: number(f[3], 4)?,
        error_low: number(f[4], 4)?,
    };
    let property = Property { id: None, machine_id: 0, code, name, unit, decimals, sort_order: 0, active: true, limits };

    let bins = machine.bin_count as usize;
    let header = lines.get(5).ok_or_else(|| parse_err(5, "missing header"))?;
    if header.split(';').count() != FIXED_COLUMNS.len() + bins || !header.starts_with(FIXED_COLUMNS[0]) {
        return Err(parse_err(5, "header does not match bin count"));
    }
    let mut scans = Vec::with_capacity(lines.len().saturating_sub(6));
    for (i, line) in lines.iter().enumerate().skip(6) {
        if line.is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split(';').collect();
        if c.len() != FIXED_COLUMNS.len() + bins {
            return Err(parse_err(i, "wrong column count"));
        }
        let values = c[FIXED_COLUMNS.len()..]
            .iter()
            .map(|v| if v.trim().is_empty() { Ok(None) } else { number(v, i).map(Some) })
            .collect::<AppResult<Vec<_>>>()?;
        scans.push(Scan {
            number: integer(c[0], i)?,
            started_at: datetime(c[1], i)?,
            ended_at: datetime(c[2], i)?,
            direction: ScanDirection::parse(c[3]).map_err(|_| parse_err(i, "invalid direction"))?,
            speed: number(c[4], i)?,
            md_position_m: number(c[5], i)?,
            edge_front_mm: number(c[6], i)?,
            edge_back_mm: number(c[7], i)?,
            sheet_break: integer::<u8>(c[8], i)? != 0,
            sensor_status: SensorStatus::from_code(integer(c[9], i)?).map_err(|_| parse_err(i, "invalid sensor status"))?,
            sensor_temp_c: number(c[10], i)?,
            values,
        });
    }
    let reel = Reel { reel_id, machine, property, scans };
    reel.validate()?;
    Ok(reel)
}

pub struct ScanCsvStorage;

impl ReelStorage for ScanCsvStorage {
    fn write(&self, reel: &Reel, dir: &Path) -> AppResult<PathBuf> {
        fs::create_dir_all(dir).map_err(AppError::io)?;
        let name = file_name(reel);
        let path = dir.join(&name);
        // Grava em arquivo temporário e renomeia para evitar arquivos parciais.
        let tmp = dir.join(format!(".{name}.tmp"));
        {
            let mut file = fs::File::create(&tmp).map_err(AppError::io)?;
            file.write_all(render(reel).as_bytes()).map_err(AppError::io)?;
            file.sync_all().map_err(AppError::io)?;
        }
        fs::rename(&tmp, &path).map_err(AppError::io)?;
        Ok(path)
    }

    fn read(&self, path: &Path) -> AppResult<Reel> {
        let bytes = fs::read(path).map_err(AppError::io)?;
        parse(&String::from_utf8_lossy(&bytes))
    }

    fn remove(&self, files: &[PathBuf], dir: &Path) -> AppResult<()> {
        for f in files {
            match fs::remove_file(f) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(AppError::io(e)),
            }
        }
        // Só remove a pasta se ela ficou vazia (nunca apaga arquivos que não foram gerados aqui).
        if fs::read_dir(dir).map(|mut d| d.next().is_none()).unwrap_or(false) {
            fs::remove_dir(dir).map_err(AppError::io)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::generator::{generate_frames, generate_reel, GeneratorParams, UniformSource};
    use domain::machine::DEFAULT_BIN_COUNT;
    use domain::property::DEFAULT_PROPERTIES;

    struct Lcg(u64);
    impl UniformSource for Lcg {
        fn next_f64(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn reel() -> Reel {
        let machine = Machine {
            id: Some(1),
            name: "PM1".into(),
            scanner_model: ScannerModel::Np1200,
            width_mm: 8000.0,
            bin_count: DEFAULT_BIN_COUNT,
            speed_m_min: 900.0,
            scan_time_s: 20.5,
        };
        let property = DEFAULT_PROPERTIES[0].to_property(1, 0);
        let start = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap().and_hms_opt(8, 0, 0).unwrap();
        let params = GeneratorParams { sheet_break_probability: 0.05, sensor_fault_probability: 0.05, ..Default::default() };
        let mut rng = Lcg(3);
        let frames = generate_frames(&machine, start, 40, &params, &mut rng).unwrap();
        generate_reel("PM1-001", &machine, &property, &frames, &params, &mut rng).unwrap()
    }

    #[test]
    fn round_trip_preserves_reel() {
        let original = reel();
        let text = render(&original);
        assert!(text.starts_with("#Format;sim-heatmaps-scan;1\r\n"));
        assert!(!text.starts_with('\u{feff}'));
        let parsed = parse(&text).unwrap();
        assert_eq!(parsed.reel_id, original.reel_id);
        assert_eq!(parsed.machine.name, "PM1");
        assert_eq!(parsed.machine.scan_time_s, 20.5);
        assert_eq!(parsed.property.limits, original.property.limits);
        assert_eq!(parsed.scans, original.scans);
    }

    #[test]
    fn rejects_foreign_or_truncated_files() {
        assert!(parse("AL300 SAMPLE").is_err());
        let text = render(&reel());
        let broken: String = text.lines().take(7).map(|l| format!("{}\r\n", &l[..l.len() / 2])).collect();
        assert!(parse(&broken).is_err());
    }

    #[test]
    fn storage_writes_and_removes_only_its_files() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("PM1").join("PM1-001");
        let r = reel();
        let path = ScanCsvStorage.write(&r, &target).unwrap();
        assert_eq!(path.file_name().unwrap(), "PM1-001_BW.csv");
        assert_eq!(ScanCsvStorage.read(&path).unwrap().scans.len(), 40);
        fs::write(target.join("notes.txt"), "keep").unwrap();
        ScanCsvStorage.remove(&[path.clone()], &target).unwrap();
        assert!(!path.exists());
        assert!(target.join("notes.txt").exists());
    }
}
