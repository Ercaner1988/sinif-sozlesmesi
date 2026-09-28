//! Kişi veri sınıfı — yol haritasının 1. adımının sorduğu "Kişi = yazar mı,
//! taraf mı?" sorusunun cevabı.
//!
//! Karar: ne biri ne öteki tek başına. `Kisi` paylaşılan bir kimlik
//! (ad-soyad + kararlı bir anahtar); "yazar" (Zotero/zopay tarafı) ve "taraf"
//! (UYAP/agent-reach tarafı) bu kimliğe **eklenen roller**dir, tek bir seçim
//! değil. Aynı gerçek kişi (ör. hem akademik yazar hem bir davada vekil olan
//! biri) iki rolü aynı anda taşıyabilir; şema bunu dışlamaz.
//!
//! Bu, ADR 0001'in "her araç kendi verisini kendi sınıf şemasıyla kodlar"
//! ilkesiyle uyumludur: alan-özel veri (ORCID, dava no) role gömülür, ortak
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

/// UYAP/agent-reach tarafındaki dava taraf türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TarafTuru {
    Davaci,
    Davali,
    Vekil,
    Muduhil,
}

/// Bir kişinin tek bir bağlamdaki (eser ya da dava) rolü.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tur")]
pub enum KisiRolu {
    Yazar {
        eser_anahtari: String,
        katki: YazarKatkisi,
    },
    Taraf {
        dava_no: String,
        taraf_turu: TarafTuru,
    },
}

/// Paylaşılan kişi kimliği. `id` araçlar arasında kararlı bir anahtardır
/// (ör. Zotero creatorID + UYAP kimlik eşlemesi el-Fihrist'te tutulur — bu
/// crate eşlemenin kendisini taşımaz, yalnız biçimini tanımlar).
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
                dava_no: "2026/123".into(),
                taraf_turu: TarafTuru::Vekil,
            });
        assert!(k.yazar_mi());
        assert!(k.taraf_mi());
        assert_eq!(k.roller.len(), 2);
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
            dava_no: "2026/1".into(),
            taraf_turu: TarafTuru::Davaci,
        });
        let json = serde_json::to_string(&k).unwrap();
        assert!(json.contains("\"tur\":\"Taraf\""));
        let geri: Kisi = serde_json::from_str(&json).unwrap();
        assert_eq!(geri, k);
    }
}
