//! Paylaşılan veri sınıfı sözleşmesi — yol haritasının 1. adımı.
//!
//! Bu crate, Ercan'ın araç ekosisteminde (zopay, zopay-word, agent-reach-rs,
//! pasli-beyin, el-Fihrist) birden çok aracın anlamca aynı veriyi (ör. bir
//! kişi) tutarlı biçimde kodlaması için üç şey sağlar:
//!
//! - [`dal`] — dal kimliği: bir değişikliği üreten YZ oturumuna bağlar.
//! - [`gunluk`] — değişiklik günlüğü: yalnız-ekleme olay kaydı biçimi.
//! - [`kisi`] — ilk paylaşılan veri sınıfı: Kişi (yazar + taraf rolleri).
//!
//! Bilinçli olarak taşımadığı şey: birleştirme (merge) mantığı. Çakışan
//! dalların uzlaştırılması AgentGit yöntemiyle ayrı bir adımda çözülecek
//! (bkz. el-Fihrist `docs/adr/0001-arac-veritabanlari-oturum-dallari.md`).

pub mod dal;
pub mod gunluk;
pub mod kisi;

pub use dal::DalKimligi;
pub use gunluk::{Gunluk, Islem, Olay};
pub use kisi::{Kisi, KisiRolu, TarafTuru, YazarKatkisi};
