//! Kişi veri sınıfı — yol haritasının 1. adımının sorduğu "Kişi = yazar mı,
//! taraf mı?" sorusunun cevabı.
//!
//! Karar: ne biri ne öteki tek başına. `Kisi` paylaşılan bir kimlik
//! (ad-soyad + kararlı bir anahtar); "yazar" (Zotero/zopay tarafı) ve "taraf"
//! (arabuluculuk dosyası tarafı) bu kimliğe **eklenen roller**dir, tek bir seçim
//! değil. Aynı gerçek kişi (ör. hem akademik yazar hem bir arabuluculuk dosyasında vekil olan
//! biri) iki rolü aynı anda taşıyabilir; şema bunu dışlamaz.
//!
//! Bu, ADR 0001'in "her araç kendi verisini kendi sınıf şemasıyla kodlar"
//! ilkesiyle uyumludur: alan-özel veri (ORCID, dosya no) role gömülür, ortak
//! kimliğe sızmaz.

use serde::{Deserialize, Serialize};

/// Zotero/zopay tarafındaki yazarlık katkı türleri (creatorType'ın alt kümesi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YazarKatkisi {
    Yazar,
    Editor,
    Cevirmen,
    KatkidaBulunan,
}

/// Bir uyuşmazlığın iki yanı. Gerçek arabuluculuk tutanaklarından alındı
/// (Taraf 1 = Başvurucu, Taraf 2 = Diğer Taraf); mahkeme kararlarındaki
/// Davacı/Davalı gibi türler UYAP karar kanalı kodlanınca, ihtiyaç çıkınca eklenir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TarafTuru {
    Basvurucu,
    DigerTaraf,
}

/// Bir kişinin tek bir bağlamdaki (eser ya da uyuşmazlık dosyası) rolü.
///
/// Vekil taraf değildir: bir tarafı temsil eder (tutanaklarda her tarafın
/// altında ayrı "Vekili" alanı). Arabulucu da üçüncü, ayrı bir roldür.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tur")]
#[non_exhaustive]
pub enum KisiRolu {
    Yazar {
        eser_anahtari: String,
        katki: YazarKatkisi,
    },
    Taraf {
        dosya_no: String,
        taraf_turu: TarafTuru,
    },
    Vekil {
        dosya_no: String,
        /// Temsil edilen tarafın `Kisi::id`'si.
        temsil_edilen: String,
    },
    Arabulucu {
        dosya_no: String,
        sicil_no: String,
    },
}

/// Kişi kaydı. `id`, kaydı üreten aracın kendi kimliğidir (ör. Zotero
/// creatorID); iki aracın aynı gerçek kişiyi aynı `id` ile anacağı varsayılmaz.
/// Araçlar arası eşleme burada değil, el-Fihrist'te isteğe bağlı ayrı bir
/// tabloda tutulur — aynı ad, farklı kişi olabileceği için otomatik birleştirme yok.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kisi {
    pub id: String,
    pub ad_soyad: String,
    pub roller: Vec<KisiRolu>,
}

impl Kisi {
    pub fn yeni(id: impl Into<String>, ad_soyad: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ad_soyad: ad_soyad.into(),
            roller: Vec::new(),
        }
    }

    pub fn rol_ekle(mut self, rol: KisiRolu) -> Self {
        self.roller.push(rol);
        self
    }

    pub fn yazar_mi(&self) -> bool {
        self.roller.iter().any(|r| matches!(r, KisiRolu::Yazar { .. }))
    }

    pub fn taraf_mi(&self) -> bool {
        self.roller.iter().any(|r| matches!(r, KisiRolu::Taraf { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tek_rol_ile_kurulan_kisi_yalniz_o_rolu_tasir() {
        let k = Kisi::yeni("K1", "Ali Ekber Çınar").rol_ekle(KisiRolu::Yazar {
            eser_anahtari: "Q4IDFF78".into(),
            katki: YazarKatkisi::Yazar,
        });
        assert!(k.yazar_mi());
        assert!(!k.taraf_mi());
    }

    #[test]
    fn ayni_kisi_iki_rolu_birden_tasiyabilir() {
        let k = Kisi::yeni("K2", "Örnek Kişi")
            .rol_ekle(KisiRolu::Yazar {
                eser_anahtari: "ABCDEFGH".into(),
                katki: YazarKatkisi::KatkidaBulunan,
            })
            .rol_ekle(KisiRolu::Taraf {
                dosya_no: "2026/123".into(),
                taraf_turu: TarafTuru::Basvurucu,
            });
        assert!(k.yazar_mi());
        assert!(k.taraf_mi());
        assert_eq!(k.roller.len(), 2);
    }

    #[test]
    fn vekil_taraf_sayilmaz_ve_temsil_ettigini_tasir() {
        let vekil = Kisi::yeni("V1", "Vekil Kişi").rol_ekle(KisiRolu::Vekil {
            dosya_no: "2026/123".into(),
            temsil_edilen: "K5".into(),
        });
        assert!(!vekil.taraf_mi());
        let json = serde_json::to_string(&vekil).unwrap();
        assert!(json.contains("\"tur\":\"Vekil\""));
        assert!(json.contains("\"temsil_edilen\":\"K5\""));
        assert_eq!(serde_json::from_str::<Kisi>(&json).unwrap(), vekil);
    }

    #[test]
    fn arabulucu_ayri_roldur_ne_taraf_ne_vekil() {
        let a = Kisi::yeni("A1", "Arabulucu Kişi").rol_ekle(KisiRolu::Arabulucu {
            dosya_no: "2026/123".into(),
            sicil_no: "12345".into(),
        });
        assert!(!a.taraf_mi());
        assert!(!a.yazar_mi());
        assert!(matches!(a.roller[0], KisiRolu::Arabulucu { .. }));
    }

    #[test]
    fn rolsuz_kisi_ne_yazar_ne_taraftir() {
        let k = Kisi::yeni("K3", "Roller sonradan gelecek");
        assert!(!k.yazar_mi());
        assert!(!k.taraf_mi());
    }

    #[test]
    fn json_yuvarlak_seyahat_tur_etiketini_korur() {
        let k = Kisi::yeni("K4", "Test").rol_ekle(KisiRolu::Taraf {
            dosya_no: "2026/1".into(),
            taraf_turu: TarafTuru::DigerTaraf,
        });
        let json = serde_json::to_string(&k).unwrap();
        assert!(json.contains("\"tur\":\"Taraf\""));
        let geri: Kisi = serde_json::from_str(&json).unwrap();
        assert_eq!(geri, k);
    }
}
