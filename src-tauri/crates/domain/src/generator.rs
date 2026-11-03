//! Simulação dos dados do scanner. A fonte de aleatoriedade é injetada para manter o domínio puro.
//!
//! Modelo de cada valor: `target + perfil_CD(bin) + deriva_MD(scan) + borda(bin) + ruído`,
//! com amplitudes expressas em frações da meia faixa de Warning da propriedade.
//! Os metadados de cada scan (tempo, velocidade, bordas, quebras) são comuns a todas as
//! propriedades da bobina; status e temperatura são de cada sensor.

use std::f64::consts::PI;

use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::format::round_to;
use crate::machine::Machine;
use crate::property::Property;
use crate::scan::{Reel, Scan, ScanDirection, SensorStatus, MAX_SCANS};
use crate::{DomainError, DomainResult};

/// Fonte de números uniformes em `[0, 1)`.
pub trait UniformSource {
    fn next_f64(&mut self) -> f64;
}

fn normal(rng: &mut dyn UniformSource) -> f64 {
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

fn uniform(rng: &mut dyn UniformSource, low: f64, high: f64) -> f64 {
    low + (high - low) * rng.next_f64()
}

/// Parâmetros da simulação. Os padrões deixam cerca de 90% dos valores dentro do Warning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorParams {
    /// Amplitude do perfil CD persistente (ondas + streaks).
    pub cd_amplitude: f64,
    /// Desvio da deriva MD (processo AR(1) entre scans).
    pub md_amplitude: f64,
    /// Ruído aleatório por ponto.
    pub noise: f64,
    /// Desvio máximo próximo às bordas da folha.
    pub edge_amplitude: f64,
    /// Quantidade de streaks (faixas estreitas) no perfil CD.
    pub streak_count: u32,
    /// Probabilidade, por scan, de iniciar uma quebra de folha.
    pub sheet_break_probability: f64,
    /// Probabilidade, por scan e sensor, de status inválido.
    pub sensor_fault_probability: f64,
}

impl Default for GeneratorParams {
    fn default() -> Self {
        Self {
            cd_amplitude: 0.5,
            md_amplitude: 0.4,
            noise: 0.35,
            edge_amplitude: 0.8,
            streak_count: 3,
            sheet_break_probability: 0.002,
            sensor_fault_probability: 0.005,
        }
    }
}

impl GeneratorParams {
    pub fn validate(&self) -> DomainResult<()> {
        let amplitudes = [
            ("cdAmplitude", self.cd_amplitude),
            ("mdAmplitude", self.md_amplitude),
            ("noise", self.noise),
            ("edgeAmplitude", self.edge_amplitude),
        ];
        for (field, v) in amplitudes {
            if !v.is_finite() || !(0.0..=5.0).contains(&v) {
                return Err(DomainError::validation(field, "range"));
            }
        }
        for (field, v) in [
            ("sheetBreakProbability", self.sheet_break_probability),
            ("sensorFaultProbability", self.sensor_fault_probability),
        ] {
            if !v.is_finite() || !(0.0..=0.5).contains(&v) {
                return Err(DomainError::validation(field, "range"));
            }
        }
        if self.streak_count > 20 {
            return Err(DomainError::validation("streakCount", "range"));
        }
        Ok(())
    }
}

/// Metadados de um scan, comuns a todas as propriedades da bobina.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanFrame {
    pub number: u32,
    pub direction: ScanDirection,
    pub started_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
    pub speed: f64,
    pub md_position_m: f64,
    pub edge_front_mm: f64,
    pub edge_back_mm: f64,
    pub sheet_break: bool,
}

/// Gera a sequência de scans: alternância de sentido, avanço MD, bordas e quebras de folha.
pub fn generate_frames(
    machine: &Machine,
    started_at: NaiveDateTime,
    scan_count: u32,
    params: &GeneratorParams,
    rng: &mut dyn UniformSource,
) -> DomainResult<Vec<ScanFrame>> {
    machine.validate()?;
    params.validate()?;
    if scan_count == 0 || scan_count > MAX_SCANS {
        return Err(DomainError::validation("scanCount", "range"));
    }
    let scan_ms = (machine.scan_time_s * 1000.0).round() as i64;
    let trim = machine.width_mm * 0.0025;
    let (mut front, mut back) = (0.0, 0.0);
    let mut md = 0.0;
    let mut break_left = 0u32;
    let mut frames = Vec::with_capacity(scan_count as usize);
    for k in 0..scan_count {
        if break_left == 0 && rng.next_f64() < params.sheet_break_probability {
            break_left = 3 + (rng.next_f64() * 8.0) as u32;
        }
        let sheet_break = break_left > 0;
        break_left = break_left.saturating_sub(1);
        // Bordas oscilam alguns mm em torno do refile nominal.
        front = (0.9 * front + uniform(rng, -1.0, 1.0)).clamp(-trim, trim);
        back = (0.9 * back + uniform(rng, -1.0, 1.0)).clamp(-trim, trim);
        let speed = if sheet_break { 0.0 } else { round_to(machine.speed_m_min * (1.0 + 0.002 * normal(rng)), 1) };
        md += speed * machine.scan_time_s / 60.0;
        let started = started_at + Duration::milliseconds(scan_ms * k as i64);
        frames.push(ScanFrame {
            number: k + 1,
            direction: if k % 2 == 0 { ScanDirection::Forward } else { ScanDirection::Reverse },
            started_at: started,
            ended_at: started + Duration::milliseconds(scan_ms),
            speed,
            md_position_m: round_to(md, 1),
            edge_front_mm: round_to(trim + front, 1),
            edge_back_mm: round_to(machine.width_mm - trim + back, 1),
            sheet_break,
        });
    }
    Ok(frames)
}

/// Perfil CD persistente da bobina: ondas + streaks + efeito de borda (em unidades da propriedade).
fn cd_profile(centers: &[f64], width: f64, h: f64, params: &GeneratorParams, rng: &mut dyn UniformSource) -> Vec<f64> {
    let waves: Vec<(f64, f64, f64)> = [0.5, 0.3, 0.2]
        .iter()
        .map(|w| {
            let cycles = uniform(rng, 0.5, 6.0);
            (w * params.cd_amplitude * h, cycles, uniform(rng, 0.0, 2.0 * PI))
        })
        .collect();
    let streaks: Vec<(f64, f64, f64)> = (0..params.streak_count)
        .map(|_| {
            let sign = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
            let amp = sign * uniform(rng, 0.6, 1.2) * params.cd_amplitude * h;
            (amp, uniform(rng, 0.05, 0.95) * width, uniform(rng, 0.002, 0.006) * width)
        })
        .collect();
    let decay = 0.03 * width;
    let edge_front = uniform(rng, -1.0, 1.0) * params.edge_amplitude * h;
    let edge_back = uniform(rng, -1.0, 1.0) * params.edge_amplitude * h;
    centers
        .iter()
        .map(|&x| {
            let wave: f64 = waves.iter().map(|(a, c, p)| a * (2.0 * PI * c * x / width + p).sin()).sum();
            let streak: f64 = streaks.iter().map(|(a, c, s)| a * (-((x - c) / s).powi(2) / 2.0).exp()).sum();
            let edge = edge_front * (-x / decay).exp() + edge_back * (-(width - x) / decay).exp();
            wave + streak + edge
        })
        .collect()
}

/// Gera a bobina de uma propriedade sobre os scans já definidos.
pub fn generate_reel(
    reel_id: &str,
    machine: &Machine,
    property: &Property,
    frames: &[ScanFrame],
    params: &GeneratorParams,
    rng: &mut dyn UniformSource,
) -> DomainResult<Reel> {
    property.validate()?;
    let limits = property.limits;
    let h = limits.warning_half_band();
    let centers = machine.bin_centers();
    let profile = cd_profile(&centers, machine.width_mm, h, params, rng);
    let phi: f64 = 0.95;
    let md_sigma = params.md_amplitude * h;
    let mut drift = normal(rng) * md_sigma;
    let mut level = 0.0;
    let mut temp = uniform(rng, 32.0, 38.0);
    let mut was_break = false;

    let mut scans = Vec::with_capacity(frames.len());
    for f in frames {
        drift = phi * drift + (1.0 - phi * phi).sqrt() * md_sigma * normal(rng);
        if was_break && !f.sheet_break {
            // Após a quebra o processo retoma em outro patamar.
            level = normal(rng) * 0.3 * md_sigma;
        }
        was_break = f.sheet_break;
        temp = (temp + 0.05 * normal(rng)).clamp(25.0, 45.0);
        let sensor_status = if rng.next_f64() < params.sensor_fault_probability {
            match (rng.next_f64() * 3.0) as u8 {
                0 => SensorStatus::NotCalibrated,
                1 => SensorStatus::Dirty,
                _ => SensorStatus::Fault,
            }
        } else {
            SensorStatus::Ok
        };
        let valid = !f.sheet_break && sensor_status == SensorStatus::Ok;
        let values = centers
            .iter()
            .zip(&profile)
            .map(|(&x, &cd)| {
                if !valid || x < f.edge_front_mm || x > f.edge_back_mm {
                    return None;
                }
                let v = round_to(limits.target + cd + drift + level + params.noise * h * normal(rng), property.decimals);
                Some(if v == 0.0 { 0.0 } else { v })
            })
            .collect();
        scans.push(Scan {
            number: f.number,
            direction: f.direction,
            started_at: f.started_at,
            ended_at: f.ended_at,
            speed: f.speed,
            md_position_m: f.md_position_m,
            edge_front_mm: f.edge_front_mm,
            edge_back_mm: f.edge_back_mm,
            sheet_break: f.sheet_break,
            sensor_status,
            sensor_temp_c: round_to(temp, 1),
            values,
        });
    }
    let reel = Reel { reel_id: reel_id.to_string(), machine: machine.clone(), property: property.clone(), scans };
    reel.validate()?;
    Ok(reel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{ScannerModel, DEFAULT_BIN_COUNT};
    use crate::property::DEFAULT_PROPERTIES;
    use chrono::NaiveDate;

    struct Lcg(u64);
    impl UniformSource for Lcg {
        fn next_f64(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn machine() -> Machine {
        Machine {
            id: Some(1),
            name: "PM1".into(),
            scanner_model: ScannerModel::Np800,
            width_mm: 8000.0,
            bin_count: DEFAULT_BIN_COUNT,
            speed_m_min: 900.0,
            scan_time_s: 30.0,
        }
    }

    fn start() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 2).unwrap().and_hms_opt(8, 0, 0).unwrap()
    }

    #[test]
    fn frames_advance_in_time_and_md() {
        let f = generate_frames(&machine(), start(), 120, &GeneratorParams::default(), &mut Lcg(1)).unwrap();
        assert_eq!(f.len(), 120);
        assert_eq!(f[1].started_at - f[0].started_at, Duration::seconds(30));
        assert_eq!(f[0].direction, ScanDirection::Forward);
        assert_eq!(f[1].direction, ScanDirection::Reverse);
        assert!(f.windows(2).all(|w| w[1].md_position_m >= w[0].md_position_m));
        assert!(f.iter().all(|s| s.edge_front_mm > 0.0 && s.edge_back_mm < 8000.0));
    }

    #[test]
    fn most_values_fall_inside_warning() {
        let m = machine();
        let p = DEFAULT_PROPERTIES[0].to_property(1, 0);
        let params = GeneratorParams { sheet_break_probability: 0.0, sensor_fault_probability: 0.0, ..Default::default() };
        let mut rng = Lcg(42);
        let frames = generate_frames(&m, start(), 200, &params, &mut rng).unwrap();
        let mut ok = 0usize;
        let mut total = 0usize;
        for seed in 0..5 {
            let reel = generate_reel("R1", &m, &p, &frames, &params, &mut Lcg(seed)).unwrap();
            for v in reel.scans.iter().flat_map(|s| s.values.iter().flatten()) {
                total += 1;
                if (p.limits.warning_low..=p.limits.warning_high).contains(v) {
                    ok += 1;
                }
            }
        }
        let ratio = ok as f64 / total as f64;
        assert!((0.8..0.98).contains(&ratio), "ratio={ratio}");
    }

    #[test]
    fn edges_and_breaks_leave_empty_bins() {
        let m = machine();
        let p = DEFAULT_PROPERTIES[1].to_property(1, 1);
        let params = GeneratorParams { sheet_break_probability: 0.5, ..Default::default() };
        let frames = generate_frames(&m, start(), 60, &params, &mut Lcg(7)).unwrap();
        let reel = generate_reel("R1", &m, &p, &frames, &params, &mut Lcg(8)).unwrap();
        let broken: Vec<_> = reel.scans.iter().filter(|s| s.sheet_break).collect();
        assert!(!broken.is_empty());
        assert!(broken.iter().all(|s| s.values.iter().all(Option::is_none)));
        let valid = reel.scans.iter().find(|s| s.is_valid()).unwrap();
        assert!(valid.values[0].is_none() && valid.values[799].is_none());
        assert!(valid.values[400].is_some());
    }

    #[test]
    fn rejects_invalid_requests() {
        let params = GeneratorParams { noise: -1.0, ..Default::default() };
        assert!(generate_frames(&machine(), start(), 10, &params, &mut Lcg(1)).is_err());
        assert!(generate_frames(&machine(), start(), 0, &GeneratorParams::default(), &mut Lcg(1)).is_err());
        assert!(generate_frames(&machine(), start(), MAX_SCANS + 1, &GeneratorParams::default(), &mut Lcg(1)).is_err());
    }
}
