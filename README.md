# Mola

Sistem tepsisinde çalışan su içme ve kalkıp yürüme hatırlatıcısı (Tauri 2 + React). Türkçe ve İngilizce.

## Nasıl çalışır

- **Su:** Her `waterIntervalMin` dakikada sağ altta bir pencere açılır. "İçtim" günlük toplama bir bardak ekler.
- **Hareket:** Klavye ve fare kullanımına göre kesintisiz oturma süresi sayılır. `idleResetMin` dakika boşta kalınırsa
  kalktın sayılır ve sayaç sıfırlanır. Süre dolunca mola penceresi açılır, "Molaya başla" geri sayımı başlatır.
- **Erteleme:** Her hatırlatma en fazla `maxSnoozes` kez ertelenebilir. Hak, hatırlatma tamamlanınca yenilenir.
- Ayarlar ve günlük istatistikler `%APPDATA%\app.mola.desktop\` altında JSON olarak saklanır.

## Geliştirme

```sh
npm install
npm run tauri dev               # geliştirme modunda çalıştır
cd src-tauri && cargo test      # zamanlayıcı testleri
npm run tauri build             # kurulum dosyası (src-tauri/target/release/bundle/nsis)
```

Kod yapısı:

- `src-tauri/src/engine.rs`: zamanlayıcı mantığı (saf, test edilebilir)
- `src-tauri/src/store.rs`: ayarlar, geçmiş, JSON kalıcılığı
- `src-tauri/src/i18n.rs`: tepsi menüsü metinleri (tr/en)
- `src-tauri/src/idle.rs`: Windows boşta kalma süresi (`GetLastInputInfo`)
- `src-tauri/src/lib.rs`: tepsi, pencereler, komutlar, saniyelik döngü
- `src/Dashboard.tsx`: panel ve ayarlar
- `src/Reminder.tsx`: hatırlatma penceresi
- `src/i18n.ts`: arayüz çevirileri (tr/en)
- `design/icon.svg`: ikon kaynağı (`npx tauri icon design/icon.png` ile tüm boyutlar üretilir)

## Lisans

MIT. Ayrıntılar için [LICENSE](LICENSE).
