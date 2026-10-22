<p align="center">
  <img src=".github/assets/banner.svg" alt="Vega, hızlı, sade ve açık kaynak Minecraft launcher" width="100%">
</p>

<p align="center">
  <a href="https://github.com/matrixdurden/vega/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/matrixdurden/vega/ci.yml?branch=main&style=flat-square&label=CI&color=7f77dd" alt="CI"></a>
  <a href="https://github.com/matrixdurden/vega/releases"><img src="https://img.shields.io/github/v/release/matrixdurden/vega?include_prereleases&style=flat-square&color=7f77dd&label=sürüm" alt="Sürüm"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/lisans-GPL--3.0-7f77dd?style=flat-square" alt="Lisans: GPL-3.0"></a>
  <img src="https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
  <img src="https://img.shields.io/badge/Rust-2024-f74c00?style=flat-square&logo=rust&logoColor=white" alt="Rust 2024">
  <img src="https://img.shields.io/badge/Svelte-5-ff3e00?style=flat-square&logo=svelte&logoColor=white" alt="Svelte 5">
</p>

<p align="center">
  <a href="README.md">English</a> · <b>Türkçe</b>
</p>

---

Vega, ayağına dolanmayan bir Minecraft launcher'ı. Sürümünü seç, Modrinth'ten birkaç mod ekle, **Oyna**'ya bas. Sadece eksik olanı indirir, her dosyayı doğrular, Java'sını kendisi getirir. Ağır işleri Rust yaptığı için de hafif kalır.

<table>
  <tr>
    <td><img src=".github/assets/screenshot-home.jpg" alt="Vega ana ekranı"></td>
    <td><img src=".github/assets/screenshot-mods.jpg" alt="Vega mod ekranı"></td>
  </tr>
</table>

## Özellikler

- **Tek tıkla Fabric.** Profiller Fabric sürümünü sabitler, kimse habersiz bir şey değiştirmez.
- **Dahili Modrinth.** Mod ara, kur, kaldır. Fabric API gibi gerekli bağımlılıklar otomatik gelir.
- **Java dert değil.** Her Minecraft sürümünün istediği Java'yı Mojang'ın kendi derlemelerinden kurar.
- **Hızlı ve doğrulanmış indirme.** 32 paralel indirme, kaynağın hash yayınladığı her dosyada SHA-1 kontrolü, otomatik yeniden deneme, yarım dosya bırakmayan yazma.
- **Hiçbir şey iki kez inmez.** Kütüphaneler, asset'ler ve Java profiller arasında paylaşılır. 1.21'de on profil, tek indirme demek.
- **Oynarken ortadan kaybolur.** Oyun açıkken launcher kendini gizler ve animasyonlarını durdurur.
- **Hafif.** Rust çekirdek ve sistemin web görünümünde çalışan Svelte 5 arayüz. İçinde koca bir tarayıcı taşımaz.
- **Türkçe ve İngilizce.**

## İndir

Windows, macOS ve Linux kurulum dosyaları v0.1.0 ile birlikte [sürümler sayfasında](https://github.com/matrixdurden/vega/releases) yayınlanacak.

> [!NOTE]
> Vega şu an **offline hesap** kullanıyor. Microsoft girişi yol haritasında sıradaki iş ve Mojang'ın API onayını bekliyor. Lütfen oyunu satın al: offline mod geliştirme ve oyuna zaten sahip olanlar için var.

## Yol haritası

- [x] Vanilla ve Fabric profilleri
- [x] Bağımlılıklarıyla birlikte Modrinth arama ve kurulum
- [x] Otomatik Java
- [x] Oyun konsolu ve çökme bildirimi
- [ ] Microsoft hesabıyla giriş
- [ ] NeoForge ve Quilt
- [ ] Modrinth mod paketleri (`.mrpack`)
- [ ] Neyin bozulduğunu açıklayan çökme analizcisi
- [ ] Profil yedekleri ve tek tıkla geri alma
- [ ] Otomatik güncelleme

## Kaynaktan derleme

[Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) 24 ve platformuna göre [Tauri gereksinimleri](https://v2.tauri.app/start/prerequisites/) gerekiyor. Debian veya Ubuntu'da:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Sonra:

```bash
git clone https://github.com/matrixdurden/vega.git
cd vega
npm install
npm run tauri dev      # uygulamayı canlı yenilemeyle çalıştırır
npm run tauri build    # kurulum dosyası üretir
```

Sadece arayüz üzerinde mi çalışıyorsun? `npm run dev` arayüzü sahte bir backend ile tarayıcıda açar, Rust gerekmez.

Launcher çekirdeğinin arayüz bağımlılığı yok, tek başına derlenip test edilebilir:

```bash
cargo test -p vega-core
# Gerçek uçtan uca deneme: Fabric 1.21.1 + Sodium, sonra oyun 60 saniye çalışır
cargo run -p vega-core --example smoke -- /tmp/vega-smoke 1.21.1 sodium 60
```

## Yapı

```text
crates/vega-core/   Rust launcher çekirdeği, arayüz kodu yok
  mojang.rs         Sürüm listesi, kütüphaneler, asset'ler, kurallar
  fabric.rs         Fabric loader profilleri
  java.rs           Mojang'ın Java runtime'ları
  modrinth.rs       Arama ve bağımlılık çözümlemeli kurulum
  launch.rs         java komut satırını oluşturur
  net.rs            Paralel, doğrulanmış, yeniden deneyen indirme
src-tauri/          Tauri kabuğu: komutlar, olaylar, oyun süreci
src/                Svelte 5 arayüzü
```

Oyun verileri Windows'ta `%APPDATA%\vega`, macOS'ta `~/Library/Application Support/vega`, Linux'ta `~/.local/share/vega` altında durur.

## Katkı

Issue ve pull request'ler açık. Pull request göndermeden önce şunları çalıştır:

```bash
cargo fmt --all && cargo clippy -p vega-core --all-targets -- -D warnings && cargo test -p vega-core
npm run check
```

## Lisans

Vega, [GNU General Public License v3.0 veya sonrası](LICENSE) ile lisanslanmış özgür bir yazılımdır.

Vega resmi bir Minecraft ürünü değildir; Mojang veya Microsoft tarafından onaylanmamıştır ve onlarla bağlantılı değildir.
