# sinif-sozlesmesi

Ercan'ın araç ekosistemi (zopay, zopay-word, agent-reach-rs, pasli-beyin,
el-Fihrist) için paylaşılan veri sınıfı şemaları, değişiklik günlüğü biçimi
ve dal kimliği. Bağlam: `ENTEGRASYON-SWOT.md` yol haritasının 1. adımı ve
el-Fihrist `docs/adr/0001-arac-veritabanlari-oturum-dallari.md`.

## Neden ayrı bir depo

ADR 0001'in kendi gerekçesi: el-Fihrist'in içinde olsaydı araçlar
el-Fihrist'e bağımlı olurdu; zopay-word'ün içinde olsaydı AGPL bulaşırdı;
her araçta bir kopya olsaydı şemalar sessizce ayrışırdı. Bu yüzden ayrı,
küçük, MIT lisanslı bir crate.

## Ne taşır, ne taşımaz

Taşır:
- `dal::DalKimligi` — bir değişikliği üreten YZ oturumuna bağlar (`AGIT_SESSION`
  varsa o, yoksa süreç kimliği).
- `gunluk::Gunluk` — yalnız-ekleme JSON Lines değişiklik günlüğü. Dal başına tek dosya (`Gunluk::dal_icin`), her kayıt `sync_data` ile kalıcı; "önce günlük, sonra durum" kuralının günlük yarısı. `
` ile kapanmamış yarım son satır kayıt sayılmaz (`Okuma::yarim_son_satir`), bir sonraki `ekle` onu budar.
- `kisi::Kisi` — ilk paylaşılan veri sınıfı. "Kişi = yazar mı, taraf mı?"
  sorusunun cevabı: ikisi de rol olarak eklenir, kimlik ortak kalır.

Taşımaz (bilinçli olarak kapsam dışı):
- Birleştirme (merge) mantığı — yol haritasının 6. adımı, ayrı.
- turso/veritabanı entegrasyonu — yol haritasının 2. adımı, tüketici
  araçların kendi işi.
- Kişi dışındaki veri sınıfları — henüz tanımlanmadı; ihtiyaç çıktıkça eklenir.

## Durum

2026-09-29: ilk sürüm. Henüz hiçbir tüketici araca (zopay, zopay-word,
pasli-beyin, el-Fihrist, agent-reach-rs) bağlanmadı — bu ayrı bir adım.
