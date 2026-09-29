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
use std::io::{self, Write};
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

/// [`Gunluk::oku`] sonucu.
#[derive(Debug)]
pub struct Okuma {
    pub olaylar: Vec<Olay>,
    /// Dosya `\n` ile kapanmamış bir son satırla bitiyor (çökmeden kalan yarım
    /// yazma). O satır `olaylar`a girmez; bir sonraki `ekle` onu budar.
    pub yarim_son_satir: bool,
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

    /// Bir dalın günlüğü: `dizin` altında o dala ait tek dosya (dal başına tek yazar).
    pub fn dal_icin(dizin: impl AsRef<Path>, dal: &DalKimligi) -> Self {
        Self::ac(dizin.as_ref().join(dal.gunluk_dosya_adi()))
    }

    /// Tek bir olayı dosyanın sonuna ekler ve diske işlenmesini bekler
    /// (`sync_data`): "önce günlük, sonra durum" kuralı ancak dönüşte kayıt
    /// kalıcıysa işler. Önceki bir çökmeden kalan, `\n` ile kapanmamış yarım son
    /// satır önce budanır — o satır hiç tamamlanmadığı için onaylanmış bir kayıt
    /// değildi.
    pub fn ekle(&self, olay: &Olay) -> io::Result<()> {
        self.yarim_kuyrugu_buda()?;
        let mut satir = serde_json::to_string(olay)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        satir.push('\n');
        let mut dosya = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.yol)?;
        dosya.write_all(satir.as_bytes())?;
        dosya.sync_data()
    }

    // ponytail: tüm dosyayı okur; günlük dal başına küçük. Büyürse sondan parça parça tara.
    fn yarim_kuyrugu_buda(&self) -> io::Result<()> {
        let bayt = match std::fs::read(&self.yol) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };
        if bayt.last().map_or(true, |&b| b == b'\n') {
            return Ok(());
        }
        let tam = bayt.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        OpenOptions::new()
            .write(true)
            .open(&self.yol)?
            .set_len(tam as u64)
    }

    /// Günlüğü baştan sona okur. Yalnız `\n` ile kapanmış satırlar kayıttır.
    /// Son satır kapanmamışsa sayılmaz ve [`Okuma::yarim_son_satir`] ile bildirilir;
    /// ortadaki bozuk bir satır ise hatadır — sessizce atlanmaz.
    pub fn oku(&self) -> io::Result<Okuma> {
        let bayt = match std::fs::read(&self.yol) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        let yarim = bayt.last().map_or(false, |&b| b != b'\n');
        let tam_uzunluk = if yarim {
            bayt.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1)
        } else {
            bayt.len()
        };
        let mut olaylar = Vec::new();
        for satir in bayt[..tam_uzunluk].split(|&b| b == b'\n') {
            if satir.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            olaylar.push(
                serde_json::from_slice(satir)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }
        Ok(Okuma {
            olaylar,
            yarim_son_satir: yarim,
        })
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

        let okuma = gunluk.oku().unwrap();
        assert!(!okuma.yarim_son_satir);
        let olaylar = okuma.olaylar;
        assert_eq!(olaylar.len(), 2);
        assert_eq!(olaylar[0].islem, Islem::Ekle);
        assert_eq!(olaylar[1].islem, Islem::Guncelle);
        assert_eq!(olaylar[0].kayit_id, "K1");
    }

    #[test]
    fn hic_yazilmamis_dosya_bos_liste_doner() {
        let dizin = tempfile::tempdir().unwrap();
        let gunluk = Gunluk::ac(dizin.path().join("yok.jsonl"));
        let okuma = gunluk.oku().unwrap();
        assert_eq!(okuma.olaylar.len(), 0);
        assert!(!okuma.yarim_son_satir);
    }

    fn ornek(kayit_id: &str) -> Olay {
        Olay::yeni(
            DalKimligi::ham("d"),
            "kisi",
            kayit_id,
            Islem::Ekle,
            serde_json::json!({}),
        )
    }

    #[test]
    fn yarim_son_satir_bildirilir_ve_kayit_sayilmaz() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("yarim.jsonl");
        let gunluk = Gunluk::ac(&yol);
        gunluk.ekle(&ornek("K1")).unwrap();
        // çökme taklidi: ikinci kayıt yarıda kesildi, "\n" hiç yazılmadı
        let mut bayt = std::fs::read(&yol).unwrap();
        bayt.extend_from_slice(br#"{"dal":"d","veri_sinifi":"ki"#);
        std::fs::write(&yol, bayt).unwrap();

        let okuma = gunluk.oku().unwrap();
        assert!(okuma.yarim_son_satir);
        assert_eq!(okuma.olaylar.len(), 1);
        assert_eq!(okuma.olaylar[0].kayit_id, "K1");
    }

    #[test]
    fn tamam_ama_yeni_satirsiz_son_kayit_da_onaysiz_sayilir() {
        // ekle her kaydı "\n" ile birlikte tek yazışta yazar; "\n"siz kalan
        // satır, JSON'u geçerli olsa bile, tamamlanmamış yazmadır.
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("k.jsonl");
        let tam = serde_json::to_string(&ornek("K1")).unwrap();
        std::fs::write(&yol, tam).unwrap();
        let okuma = Gunluk::ac(&yol).oku().unwrap();
        assert!(okuma.yarim_son_satir);
        assert_eq!(okuma.olaylar.len(), 0);
    }

    #[test]
    fn ekle_yarim_kuyrugu_budar_ve_oncekilere_dokunmaz() {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join("buda.jsonl");
        let gunluk = Gunluk::ac(&yol);
        gunluk.ekle(&ornek("K1")).unwrap();
        let mut bayt = std::fs::read(&yol).unwrap();
        bayt.extend_from_slice(b"{yarim");
        std::fs::write(&yol, bayt).unwrap();

        gunluk.ekle(&ornek("K2")).unwrap();

        let okuma = gunluk.oku().unwrap();
        assert!(!okuma.yarim_son_satir);
        let idler: Vec<_> = okuma.olaylar.iter().map(|o| o.kayit_id.as_str()).collect();
        assert_eq!(idler, ["K1", "K2"]);
    }

    #[test]
    fn dal_icin_her_dala_ayri_dosya_verir() {
        let dizin = tempfile::tempdir().unwrap();
        let a = Gunluk::dal_icin(dizin.path(), &DalKimligi::ham("x/y@z"));
        let b = Gunluk::dal_icin(dizin.path(), &DalKimligi::ham("x/y@w"));
        a.ekle(&ornek("A")).unwrap();
        b.ekle(&ornek("B")).unwrap();
        assert_eq!(a.oku().unwrap().olaylar[0].kayit_id, "A");
        assert_eq!(b.oku().unwrap().olaylar[0].kayit_id, "B");
        assert_eq!(std::fs::read_dir(dizin.path()).unwrap().count(), 2);
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
