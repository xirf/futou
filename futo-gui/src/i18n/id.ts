const id: Record<string, string> = {
  "app.title": "Futou",
  "app.subtitle": "{count} service, {activeCount} aktif",
  "app.daemonOn": "ON",
  "app.daemonOff": "OFF",
  "app.add": "Tambah",
  "app.settings": "Pengaturan",
  "app.back": "Kembali",
  "app.save": "Simpan",
  "app.saved": "Tersimpan",

  "settings.title": "Pengaturan",
  "settings.language": "Bahasa",
  "settings.languageEn": "Inggris",
  "settings.languageId": "Indonesia",
  "settings.installLocation": "Lokasi Instalasi",
  "settings.installLocationDesc":
    "Direktori tempat runtime (PHP, Node.js, MariaDB, dsb.) disimpan.",
  "settings.browse": "Pilih",
  "settings.daemonAutostart": "Jalankan daemon saat startup",
  "settings.daemonAutostartDesc":
    "Otomatis menjalankan daemon futou saat Anda login ke Windows.",

  "service.start": "Mulai",
  "service.stop": "Hentikan",
  "service.restart": "Mulai Ulang",
  "service.config": "Konfigurasi",
  "service.openDir": "Buka direktori",
  "service.uninstall": "Hapus",
  "service.switching": "Mengganti versi...",
  "service.noRuntimes": "Belum ada runtime terinstal.",

  "modal.title": "Tambah Service",
  "modal.searchPlaceholder": "Cari service...",
  "modal.installing": "Menginstal",
  "modal.install": "Instal",
  "modal.close": "Tutup",
  "modal.noCatalogue": "Katalog kosong atau tidak tersedia.",
  "modal.noResults": "Tidak ada service yang cocok.",

  "logs.title": "Log",
  "logs.empty": "Pilih service untuk melihat log.",

  "daemon.failedStart": "Daemon gagal start",
  "daemon.starting": "Memulai daemon...",

  "error.activate": "Gagal aktivasi {runtime}: {error}",
  "error.deactivate": "Gagal deaktivasi {runtime}: {error}",
  "error.start": "Gagal start {runtime}: {error}",
  "error.stop": "Gagal stop {runtime}: {error}",
  "error.switch": "Gagal mengganti versi {runtime}: {error}",
  "error.open": "Gagal membuka: {error}",
  "error.generic": "Error: {error}",

  "confirm.switchDb": "Mengganti versi {runtime} dari {from} ke {to}.\n\nServer akan dihentikan.\n{runtimeUpper} adalah database — versi berbeda bisa menyebabkan ketidakcocokan data.\nLanjutkan?",
  "confirm.switch": "Mengganti versi {runtime} dari {from} ke {to}.\n\nServer akan dihentikan dan diaktifkan kembali dengan versi baru.\nLanjutkan?",
  "confirm.uninstall": "Hapus {runtime} {version}?\n\nSemua file akan dihapus dan tidak bisa dikembalikan.",
};

export default id;
