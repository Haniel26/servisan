# Konten Medsos - Servisan v0.2.0

Halo temen-temen! 👋

Melanjutkan update kemarin, **Servisan v0.2.0** resmi rilis nih! 🚀

Banyak yang pengen coba Servisan karena enteng banget (< 15MB RAM), tapi mager mindahin puluhan database dari Laragon/XAMPP. Di versi ini masalah itu udah beres dengan fitur baru: **Automated Database Migration**.

Tinggal buka Menu -> **Database Migration**, Servisan bakal otomatis scan semua database Laragon/XAMPP yang ada di komputer. Kerennya, ada **Zero-Touch Autopilot**: misal Laragon lagi mati atau error, Servisan bakal otomatis menyalakan instance MySQL background darurat secara silent buat transfer data. Temen-temen ga perlu buka atau utak-atik Laragon-nya sama sekali!

Rilis ini juga menggabungkan Pull Request dari mas @mteguhfh:
- Fix crash fatal error saat MySQL mati di PHP 8.1+
- Port MySQL & versi dashboard sekarang dinamis mengikuti `app.toml`
- Teks log console sekarang bisa diseleksi dan di-copy

Yang mau coba atau update, langsung cek di sini ya:
👉 https://github.com/Haniel26/servisan/releases/tag/v0.2.0

Ditunggu feedback dan bintang ⭐-nya di GitHub. Semoga bermanfaat! 🙏

#php #laravel #servisan #rust #webdeveloper #laragon #xampp #opensource
