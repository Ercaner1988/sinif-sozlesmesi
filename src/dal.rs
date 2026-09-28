//! Dal kimliği: bir veri değişikliğini onu üreten YZ oturumuna bağlar.
//!
//! Yöntem AgentGit'ten ödünç alınır (bkz. el-Fihrist docs/adr/0001): dal birimi
//! oturumdur, `AGIT_SESSION` varsa o kullanılır, yoksa sürecin kendi kimliği
//! (pid + başlangıç zamanı) düşer. Bu crate AgentGit'e derleme zamanı
//! bağımlılığı taşımaz — yalnız ortam değişkenini okur.

use std::time::{SystemTime, UNIX_EPOCH};

/// Bir veri değişikliğini üreten oturumun kimliği.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DalKimligi(String);

impl DalKimligi {
    /// Bu sürecin kendi dal kimliği: `AGIT_SESSION` varsa o, yoksa `<pid>-<başlangıç ms>`.
    pub fn yerel() -> Self {
        std::env::var("AGIT_SESSION")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(Self)
            .unwrap_or_else(Self::surec_kimligi)
    }

    fn surec_kimligi() -> Self {
        let simdi_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Self(format!("{}-{}", std::process::id(), simdi_ms))
    }

    /// Testler ve harici sistemlerden (ör. dosya adı) gelen ham kimlikten kurar.
    pub fn ham(kimlik: impl Into<String>) -> Self {
        Self(kimlik.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DalKimligi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // AGIT_SESSION süreç-geneli; testler cargo tarafından aynı süreçte paralel
    // koşulabiliyor. Kilit olmadan biri diğerinin ortam değişkenini görür.
    static KILIT: Mutex<()> = Mutex::new(());

    #[test]
    fn agit_session_varsa_onu_kullanir() {
        let _g = KILIT.lock().unwrap();
        std::env::set_var("AGIT_SESSION", "buzbey/entegrasyon-plani@grill-entegrasyon-swot");
        assert_eq!(
            DalKimligi::yerel().as_str(),
            "buzbey/entegrasyon-plani@grill-entegrasyon-swot"
        );
        std::env::remove_var("AGIT_SESSION");
    }

    #[test]
    fn agit_session_yoksa_surec_kimligine_duser() {
        let _g = KILIT.lock().unwrap();
        std::env::remove_var("AGIT_SESSION");
        let k = DalKimligi::yerel();
        assert!(k.as_str().contains('-'));
        assert!(k.as_str().starts_with(&std::process::id().to_string()));
    }

    #[test]
    fn bos_agit_session_gecersiz_sayilir() {
        let _g = KILIT.lock().unwrap();
        std::env::set_var("AGIT_SESSION", "   ");
        let k = DalKimligi::yerel();
        assert_ne!(k.as_str(), "   ");
        std::env::remove_var("AGIT_SESSION");
    }
}
