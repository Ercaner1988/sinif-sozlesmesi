//! Dışarıdan koşulan doğrulama: `cargo run --example dal_kimligi`.
//! `AGIT_SESSION` ortamda varsa onu, yoksa süreç kimliğini basar.
fn main() {
    println!("{}", sinif_sozlesmesi::DalKimligi::yerel());
}
