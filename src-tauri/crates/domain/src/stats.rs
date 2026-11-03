use serde::Serialize;

use crate::scan::Scan;

/// Estatística descritiva (desvio padrão populacional) usada no painel de informações.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub mean: f64,
    pub std_dev: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl Summary {
    pub fn of(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        Some(Self {
            mean,
            std_dev: variance.sqrt(),
            count: values.len(),
            min: values.iter().copied().fold(f64::INFINITY, f64::min),
            max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        })
    }
}

/// Perfis CD agregados sobre todos os scans: média, mínimo e máximo por bin.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdProfiles {
    pub avg: Vec<Option<f64>>,
    pub min: Vec<Option<f64>>,
    pub max: Vec<Option<f64>>,
}

pub fn cd_profiles(scans: &[Scan], bin_count: usize) -> CdProfiles {
    let mut sum = vec![0.0; bin_count];
    let mut count = vec![0usize; bin_count];
    let mut min = vec![None::<f64>; bin_count];
    let mut max = vec![None::<f64>; bin_count];
    for s in scans {
        for (i, v) in s.values.iter().enumerate().take(bin_count) {
            if let Some(v) = *v {
                sum[i] += v;
                count[i] += 1;
                min[i] = Some(min[i].map_or(v, |m| m.min(v)));
                max[i] = Some(max[i].map_or(v, |m| m.max(v)));
            }
        }
    }
    let avg = sum.iter().zip(&count).map(|(s, &c)| (c > 0).then(|| s / c as f64)).collect();
    CdProfiles { avg, min, max }
}

/// Série MD: média de cada scan (`None` em scans sem dados).
pub fn md_series(scans: &[Scan]) -> Vec<Option<f64>> {
    scans.iter().map(|s| Summary::of(&s.values.iter().flatten().copied().collect::<Vec<_>>()).map(|x| x.mean)).collect()
}

/// Estatística de todos os valores válidos da bobina.
pub fn reel_summary(scans: &[Scan]) -> Option<Summary> {
    Summary::of(&scans.iter().flat_map(|s| s.values.iter().flatten().copied()).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{ScanDirection, SensorStatus};
    use chrono::NaiveDate;

    fn scan(values: Vec<Option<f64>>) -> Scan {
        let t = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap().and_hms_opt(8, 0, 0).unwrap();
        Scan {
            number: 1,
            direction: ScanDirection::Forward,
            started_at: t,
            ended_at: t,
            speed: 900.0,
            md_position_m: 450.0,
            edge_front_mm: 0.0,
            edge_back_mm: 30.0,
            sheet_break: false,
            sensor_status: SensorStatus::Ok,
            sensor_temp_c: 35.0,
            values,
        }
    }

    #[test]
    fn population_std_dev() {
        let s = Summary::of(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert_eq!((s.mean, s.std_dev, s.count, s.min, s.max), (5.0, 2.0, 8, 2.0, 9.0));
        assert!(Summary::of(&[]).is_none());
    }

    #[test]
    fn aggregates_ignoring_empty_bins() {
        let scans = [scan(vec![Some(1.0), None, Some(3.0)]), scan(vec![Some(3.0), None, Some(5.0)]), scan(vec![None; 3])];
        let p = cd_profiles(&scans, 3);
        assert_eq!(p.avg, vec![Some(2.0), None, Some(4.0)]);
        assert_eq!(p.min, vec![Some(1.0), None, Some(3.0)]);
        assert_eq!(p.max, vec![Some(3.0), None, Some(5.0)]);
        assert_eq!(md_series(&scans), vec![Some(2.0), Some(4.0), None]);
        assert_eq!(reel_summary(&scans).unwrap().count, 4);
    }
}
