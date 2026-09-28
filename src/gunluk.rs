//! Değişiklik günlüğü: yalnız-ekleme yapılan olay kaydı.
//!
//! el-Fihrist ADR 0001'in "değişiklikler yalnız ekleme yapılan bir günlükte
//! tutulur" ilkesinin ortak biçimi. Bu crate birleştirme (merge) mantığı
//! taşımaz — o, AgentGit yöntemiyle çözülecek ayrı bir adımdır (yol
//! haritasının 6. adımı). Burada yalnız kayıt biçimi ve dosyaya yalnız-ekleme
//! yazma/okuma var.

use crate::dal::DalKimligi;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Bir veri sınıfı kaydında yapılan işlemin türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Islem {
    Ekle,
    Guncelle,
    Sil,
}

/// Günlüğe yazılan tek bir olay: hangi dal, hangi veri sınıfı, hangi işlem, ne zaman.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Olay {
    pub dal: DalKimligi,
    pub veri_sinifi: String,
    pub kayit_id: String,
    pub islem: Islem,
    pub zaman_ms: u128,
    /// Kaydın kendisi — veri sınıfına göre değişir, bu crate şemasını sabitlemez.
    pub yuk: serde_json::Value,
}

impl Olay {
    pub fn yeni(
        dal: DalKimligi,
        veri_sinifi: impl Into<String>,
        kayit_id: impl Into<String>,
        islem: Islem,
        yuk: serde_json::Value,
    ) -> Self {
        let zaman_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Self {
            dal,
            veri_sinifi: veri_sinifi.into(),
            kayit_id: kayit_id.into(),
            islem,
            zaman_ms,
            yuk,
        }
    }
}

/// Dosyaya yalnız-ekleme JSON Lines günlüğü: her satır bir [`Olay`].
pub struct Gunluk {
    yol: std::path::PathBuf,
}

impl Gunluk {
    pub fn ac(yol: impl AsRef<Path>) -> Self {
        Self {
            yol: yol.as_ref().to_path_buf(),
        }
    }

    /// Tek bir olayı dosyanın sonuna ekler. Var olan satırlara dokunmaz.
    pub fn ekle(&self, olay: &Olay) -> io::Result<()> {
        let mut dosya = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.yol)?;
        let satir = serde_json::to_string(olay)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        writeln!(dosya, "{satir}")
    }

    /// Günlüğü baştan sona okur. Bozuk bir satır varsa hata döner — sessizce atlamaz.
    pub fn oku(&self) -> io::Result<Vec<Olay>> {
        if !self.yol.exists() {
            return Ok(Vec::new());
        }
        let dosya = std::fs::File::open(&self.yol)?;
        BufReader::new(dosya)
            .lines()
            .filter(|s| s.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(true))
            .map(|satir| {
                let satir = satir?;
                serde_json::from_str(&satir)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ekle_sonra_oku_sirayla_geri_verir() {
        let dizin = tempfile::tempdir().unwrap();
        let gunluk = Gunluk::ac(dizin.path().join("olaylar.jsonl"));
        let dal = DalKimligi::ham("test-dali");

        gunluk
            .ekle(&Olay::yeni(
                dal.clone(),
                "kisi",
                "K1",
                Islem::Ekle,
                serde_json::json!({"ad_soyad": "Test Kişi"}),
            ))
            .unwrap();
        gunluk
            .ekle(&Olay::yeni(
                dal.clone(),
                "kisi",
                "K1",
                Islem::Guncelle,
                serde_json::json!({"ad_soyad": "Test Kişi (düzeltildi)"}),
            ))
            .unwrap();

        let olaylar = gunluk.oku().unwrap();
        assert_eq!(olaylar.len(), 2);
        assert_eq!(olaylar[0].islem, Islem::Ekle);
        assert_eq!(olaylar[1].islem, Islem::Guncelle);
        assert_eq!(olaylar[0].kayit_id, "K1");
    }

    #[test]
    fn hic_yazilmamis_dosya_bos_liste_doner() {
        let dizin = tempfile::tempdir().unwrap();
        let gunluk = Gunluk::ac(dizin.path().join("yok.jsonl"));
        assert_eq!(gunluk.oku().unwrap().len(), 0);
    }

    #[test]
    fn bozuk_satir_hata_dondurur_sessizce_atlamaz() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("bozuk.jsonl");
        std::fs::write(&yol, "bu json degil\n").unwrap();
        let gunluk = Gunluk::ac(&yol);
        assert!(gunluk.oku().is_err());
    }
}
