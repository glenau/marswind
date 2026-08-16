<div align="center">

<img src="../../src-tauri/icons/128x128@2x.png" alt="" width="128" height="128">

# Marswind

**Bilgisayarınızın sesi için canlı altyazı ve çeviri. Tamamen çevrimdışı.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)
[![Platform: macOS 14.4+](https://img.shields.io/badge/platform-macOS%2014.4%2B-lightgrey.svg)](#platformlar)
[![Version](https://img.shields.io/badge/version-0.1.0-brightgreen.svg)](#)

[English](../../README.md) ·
[Русский](README.ru.md) ·
[Deutsch](README.de.md) ·
[Español](README.es.md) ·
[Français](README.fr.md) ·
[Italiano](README.it.md) ·
[Português](README.pt.md) ·
[Polski](README.pl.md) ·
**Türkçe** ·
[Українська](README.uk.md) ·
[中文](README.zh.md) ·
[日本語](README.ja.md) ·
[한국어](README.ko.md)

<img src="../screenshot.png" alt="Marswind penceresi: solda özgün metin, sağda İspanyolca çevirisi" width="900">

</div>

Marswind, makinenizde çalan her şeyi - bir YouTube videosunu, bir Google Meet,
Teams ya da Zoom görüşmesini, yerel bir video dosyasını - dinler, konuşmayı
tanır ve konuşma sürerken seçtiğiniz dile çevirir.

API anahtarı yok, hesap yok, internet yok. Modeller bir kez indirilir ve sonra
yerelde çalışır; ses bellekte kalır, hiçbir zaman diske yazılmaz ve hiçbir yere
gönderilmez.

## Neler yapar

- **Sistem sesini yakalar**, sanal ses sürücüsü olmadan - makinenin çaldığı her
  şeyi ya da tarayıcı gibi tek bir uygulamayı
- **Konuşmayı tanır**, GPU üzerinde whisper.cpp ile: altyazılar okuyanın altında
  yeniden yazılmak yerine konuşma ilerledikçe uzar
- **Konuşma sürerken çevirir** - kelimeler cümle bitince değil, sabitlenir
  sabitlenmez çeviriciye gider ve çeviri kelime kelime gelir
- **Modelleri uygulama içinden yönetir**: yedi tanıma ve beş çeviri modeli,
  ilerleme göstergesi ve SHA-256 doğrulamasıyla indirilir
- **Her oturumu kaydeder** - sonradan göz atılabilir; metin, altyazı (`.srt`) ya
  da zaman bilgileriyle JSON olarak dışa aktarılabilir
- **Örnek ses klipleriyle gelir**, video aramadan denenebilsin diye
- **On üç dil konuşur** - çevirdiği dillerin aynısı - açık veya koyu temada ve
  yalnızca yazıyı değil tüm arayüzü ölçekleyen bir metin boyutuyla

### Diller

İngilizce, Rusça, Almanca, İspanyolca, Fransızca, İtalyanca, Portekizce, Lehçe,
Türkçe, Ukraynaca, Çince, Japonca ve Korece; hem hedef dil olarak hem de
pencerenin kendi dili olarak. Konuşulan dil varsayılan olarak sesten çıkarılır
ve tanıma, whisper'ın kapsadığı her şeyi kapsar.

## Nasıl çalışır

```
Sistem sesi  →  16 kHz mono'ya yeniden örnekleme  →  konuşma algılama (Silero)
             →  konuşma tanıma (whisper.cpp)
             →  çeviri (llama.cpp, ayrı bir süreçte)
             →  döküm: solda özgün metin, yanında çevirisi
```

Arayüzün altındaki her şey Rust ile yazılmıştır ve kendi iş parçacıklarında
çalışır; çeviri ayrı bir ikili dosyada yaşar, çünkü whisper.cpp ile llama.cpp
tek bir süreci paylaşamaz. Tasarım ve gerekçeleri
[docs/ARCHITECTURE.md](../ARCHITECTURE.md) içinde.

Apple Silicon üzerinde varsayılan modellerle, [tests/](../../tests/README.md)
içindeki sentetik derlem üzerinde ölçüldü - klip başına üç koşunun medyanları:
ilk altyazı klibin başlangıcından yaklaşık 6 saniye sonra, sonrakiler her 2-3
saniyede bir, ve kelime hata oranı temiz bir okumada %4, özel adlar ve sayılarla
dolu bir klipte %23. Tanıma belirlenimci değildir ve tek bir koşu yirmi puan
kadar oynar; dolayısıyla bunlar sonuç değil medyandır. Sayıların nasıl
üretildiği test düzeneğinin yanında belgelenmiştir.

## Platformlar

| Platform | Durum |
|---|---|
| **macOS 14.4+** | Destekleniyor - Core Audio process taps, Metal |
| **Windows** | Geliştiriliyor - WASAPI loopback |
| **Linux** | Geliştiriliyor - PipeWire |

Uygulama bugün Windows ve Linux'ta derleniyor ve açılıyor, ama ses yakalama
orada kendini kullanılamaz olarak bildiriyor: dinleyecek bir şeyi olmayan bir
pencere. Yakalamanın üstündeki her şey zaten platformdan bağımsız.

BlackHole gibi bir sanal ses sürücüsü **hiçbir** platformda gerekmez: yakalama
işletim sisteminin yerel API'lerinden geçer.

## Gereksinimler

| | |
|---|---|
| macOS | 14.4 veya üzeri, Apple Silicon ya da Intel |
| Bellek | Yalnız tanıma için 8 GB, çeviriyle birlikte 16 GB |
| Disk | Seçtiğiniz modeller için 0,5-4,5 GB |
| Derlemek için | [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, cmake (`brew install cmake`) |

## Kurulum

### İndir

[Son sürümde](https://github.com/glenau/marswind/releases/latest) bir `.dmg`
var. Aç, Marswind'i Applications klasörüne sürükle, bu kadar - yaklaşık 13 MB,
çünkü modeller sonradan ve yalnızca seçtiklerin iniyor.

**macOS ilk denemede açmayı reddedecek.** İmaj imzalı ama noterlenmiş değil: bu
projenin arkasında ücretli bir Developer ID sertifikası yok ve Gatekeeper böyle
olan her şeyi tanımsız sayıyor. Aşmanın yolu:

1. Uygulamayı açın ve engellenmesine izin verin. **Done**'a basın, "Move to
   Bin"e değil.
2. **Sistem Ayarları → Gizlilik ve Güvenlik**, aşağı **Güvenlik** bölümüne
   inin. Marswind'in engellendiğini söyleyen bir satır ve yanında **Yine de Aç**
   düğmesi olacak.
3. Ona basın, Touch ID veya parolayla doğrulayın ve bir kez daha onaylayın.

macOS bir kez sorar ve hatırlar. Düğme yalnızca engellenmiş bir açılıştan sonra
belirir ve yaklaşık bir saat durur; yoksa uygulamayı yeniden açın.

Uygulamaya sağ tıklayıp Aç demek bunun eski kısayoluydu ve macOS 14'te hâlâ
çalışıyor. macOS 15 bunu kaldırdı, dolayısıyla her yerde çalışan yol ayarlardan
geçen yol.

### Ya da derle

```bash
git clone https://github.com/glenau/marswind.git
cd marswind
npm install
npm run install:macos
```

Bu, çeviri işçisini derler, release paketini derler, ad-hoc imzalar ve
`/Applications/Marswind.app` konumuna kopyalar. İlk derleme birkaç dakika sürer -
whisper.cpp ve llama.cpp kaynaktan derlenir. Başka hiçbir şey gerekmiyor:
çekilecek alt modül, elle kurulacak kütüphane ya da önceden indirilecek model
yok.

### İlk çalıştırma

1. `open /Applications/Marswind.app`
2. macOS **Ses Kaydı** iznini ister. Verin - onsuz uygulama hiçbir şey duymaz.
   Reddedildiyse Sistem Ayarları → Gizlilik ve Güvenlik → Ses Kaydı üzerinden
   yeniden verilebilir.
3. **Ayarlar**'ı aç ve bir tanıma, bir çeviri modeli indir.
   `Large v3 Turbo (compressed)` ve `Qwen3 4B Instruct` 16 GB ve üstü için
   varsayılanlardır; `Small` ve `Qwen3 1.7B` 8 GB'a sığar. Yaklaşık 3 GB indirme,
   her biri yayımlanmış bir sağlama toplamıyla doğrulanır. Her satır ağırlıklarının
   lisansını yazar - bkz. [docs/MODELS.md](../MODELS.md).
4. **Dinlemeye başla**'ya bas ve konuşma içeren bir şey çal. Video aramak
   istemezsen ayarlarda dört örnek klip var.

Kendi derlediğiniz kopya hakkında iki not:

- **Ad-hoc imzalıdır.** İmza belirli bir derleme için sabittir, dolayısıyla ses
  izni korunur - ama yeniden derleme yeni bir kimlik üretir ve macOS izni tekrar
  sorar. Bunu bitiren şey bir Developer ID sertifikasıdır ve henüz yok.
- **Çalışırken uygulamayı taşımayın.** Güncellemek için `npm run install:macos`
  komutunu yeniden çalıştırın; `/Applications/Marswind.app` dosyasını yerinde
  değiştirir.

### Disk imajı oluşturma

```bash
npm run build:dmg
```

Sürüm paketini derler, imzalar ve
`src-tauri/target/Marswind-<sürüm>-<mimari>.dmg` dosyasına paketler - bir sürüme
eklenen imajın aynısı, yukarıdakiyle aynı Gatekeeper uyarısıyla. Etrafındaki
kontrol listesi [docs/RELEASING.md](../RELEASING.md) içinde.

## Geliştirme

`tauri dev`, `Info.plist` ve imza olmayan çıplak bir çalıştırılabilir üretir ve
Core Audio process tap'leri bu biçimde çalışmaz. Bunun yerine hata ayıklama
paketini derleyip imzalayan ve başlatan komutu kullanın:

```bash
npm run dev:macos
```

| Komut | Ne yapar |
|---|---|
| `npm run dev:macos` | hata ayıklama paketini derle, imzala ve başlat |
| `npm run install:macos` | release paketini derle ve kur |
| `npm run check` | Svelte ve TypeScript türleri |
| `npm run build:dmg` | başkasına verilecek imzalı bir `.dmg` |
| `npm run build:sidecar` | yalnızca çeviri işçisi |
| `npm run build:icons` | simgeyi `scripts/make-icon.py` üzerinden yeniden çiz |
| `npm run licenses` | `THIRD-PARTY-NOTICES.md` dosyasını lock dosyalarından yeniden üret |

CI yok: whisper.cpp ve llama.cpp kaynaktan derleniyor ve test düzeneği sesi
sistem çıkışından çalıyor, bu yüzden her denetim yerel bir komut.
[CONTRIBUTING.md](../../CONTRIBUTING.md) hepsini listeler.

## Testler

Birim testleri saf mantığı kapsar; [tests/](../../tests/README.md) içindeki
betikler sesi sistem çıkışından çalar ve gerçek hattan çıkanı puanlar - tanıma,
çeviri ve gecikmeyi birlikte.

```bash
npm run build:sidecar
cargo test --manifest-path src-tauri/Cargo.toml
```

İlk satır bir kez gerekir, sonrasında yalnızca `cargo clean` ardından. Tauri
çeviri işçisini sidecar olarak paketler, bu yüzden ikili dosya yokken build
betiği `src-tauri`'yi derlemeyi tümden reddeder: taze bir klonda tek başına
`cargo test`, `resource path 'binaries/marswind-translator-…' doesn't exist`
hatasında durur. Her `npm run` komutu bu adımı sizin yerinize yapar; doğrudan
`cargo` çağırmak yapmaz.

Hat betikleri derlenmiş ve imzalanmış bir paket ile kurulu modeller ister:

```bash
npm run dev:macos
tests/run-capture.sh
tests/run-asr.sh
tests/run-pipeline.sh
```

Derlem üzerindeki tek bir koşu yaklaşık yirmi puan kelime hata oranı oynar, bu
yüzden tek bir sayı tek başına bir şey ifade etmez. Koşular arasında medyanları
karşılaştırın ve yalnızca puanları değil dökümleri de okuyun.

## Gizlilik

- Ses **bellekte** yakalanır, yeniden örneklenir ve tanınır. Asla diske yazılmaz
  ve hiçbir yere gönderilmez.
- Tek ağ trafiği, istediğiniz modellerin indirilmesidir. Kurulduktan sonra
  uygulama hiç ağ trafiği üretmez.
- Telemetri yok, analitik yok, çökme raporu yok, hesap yok.
- Dökümler yalnızca uygulamanın veri klasörüne yazılır, Geçmiş görünümünün
  gösterecek bir şeyi olsun diye. Uygulama içinden silinebilirler.

## Katkı

Hata bildirimleri, fikirler ve pull request'ler memnuniyetle karşılanır.
[CONTRIBUTING.md](../../CONTRIBUTING.md) kurulumu, denetimleri, commit kuralını
ve incelemenin neye baktığını anlatır. Büyük bir şeye başlamadan önce bir issue
açın - bariz görünen birkaç iyileştirme zaten denendi ve ölçümleri kaydedilerek
geri alındı.

- [Davranış kuralları](../../CODE_OF_CONDUCT.md)
- [Güvenlik politikası](../../SECURITY.md) - güvenlik açıklarını issue'da değil,
  özel olarak bildirin

## Neyin üstüne kurulu

| | | |
|---|---|---|
| [whisper.cpp](https://github.com/ggml-org/whisper.cpp) | MIT | tanıma, ve onunla birlikte Silero VAD uygulaması |
| [llama.cpp](https://github.com/ggml-org/llama.cpp) | MIT | çeviri, kendi sürecinde |
| [ggml](https://github.com/ggml-org/ggml) | MIT | her ikisinin altındaki tensör kütüphanesi ve Metal arka ucu |
| [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) | Unlicense | whisper.cpp için Rust bağlayıcısı |
| [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) | MIT / Apache-2.0 | llama.cpp için Rust bağlayıcısı |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT | öbek sınırlarını bulan model |
| [Tauri](https://tauri.app) | MIT / Apache-2.0 | pencere ve süreç sınırı |
| [Svelte](https://svelte.dev) | MIT | arayüz |
| [rubato](https://github.com/HEnquist/rubato) | MIT | whisper'ın önündeki FFT yeniden örnekleyici |

Bağımlılık ağacının tamamı, her paketin lisansıyla birlikte,
[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) içinde - lock
dosyalarından üretiliyor ve uygulamanın içine lisansın yanına konuyor.

**Bunların hiçbiri modelleri kapsamaz.** Modeller senin isteğin üzerine
[Hugging Face](https://huggingface.co) üzerinden iniyor ve yayımcılarının
koşullarını koruyor: whisper ve Silero modelleri MIT, Qwen3 Apache-2.0, Gemma 3
ise açık kaynak lisansı olmayan ve çıktının kullanımına koşul koyan
[Google'ın kendi koşulları](https://ai.google.dev/gemma/terms) altında.
Ayarlardaki her satır, indirme başlamadan önce lisansını yazar. Ayrıntılar
[docs/MODELS.md](../MODELS.md) içinde.

## Lisans

MIT - bkz. [LICENSE](../../LICENSE). Bu, deponun kendisi içindir; modelleri
kapsamaz ve yukarıdaki bildirimler işaret ettikleri lisansların yerine geçmez.
