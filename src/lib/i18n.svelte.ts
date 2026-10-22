export type Lang = "tr" | "en";

const tr = {
  "nav.home": "Ana sayfa",
  "nav.mods": "Modlar",
  "nav.settings": "Ayarlar",

  "home.welcome": "Tekrar hoş geldin",
  "home.play": "Oyna",
  "home.stop": "Durdur",
  "home.running": "Oyun çalışıyor",
  "home.mods": "Modlar",
  "home.noMods": "Henüz mod yok",
  "home.addMod": "Ekle",
  "home.vanillaNote": "Vanilla, modsuz",
  "home.console": "Konsol",
  "home.firstRun": "İlk profil hazırlanıyor…",

  "stage.metadata": "Sürüm bilgisi",
  "stage.libraries": "Kütüphaneler",
  "stage.assets": "Ses ve dokular",
  "stage.java": "Java",
  "stage.mods": "Modlar",
  "stage.starting": "Başlatılıyor",

  "profile.new": "Yeni profil",
  "profile.openFolder": "Klasörü aç",
  "profile.delete": "Profili sil",
  "profile.confirmDelete": "Emin misin? Dünyalar da silinir",
  "profile.name": "İsim",
  "profile.version": "Minecraft sürümü",
  "profile.loader": "Mod yükleyici",
  "profile.create": "Oluştur",
  "profile.cancel": "Vazgeç",
  "profile.fabricHint": "Fabric ile mod kurabilirsin. Vanilla saf oyundur.",

  "mods.title": "Modlar",
  "mods.search": "Modrinth'te ara",
  "mods.discover": "Keşfet",
  "mods.installed": "Yüklü",
  "mods.install": "Ekle",
  "mods.added": "Yüklü",
  "mods.remove": "Kaldır",
  "mods.dependency": "bağımlılık",
  "mods.empty": "Sonuç bulunamadı",
  "mods.noneInstalled": "Bu profilde henüz mod yok. Keşfet sekmesinden ekleyebilirsin.",
  "mods.vanillaTitle": "Bu profil modsuz",
  "mods.vanillaBody": "Mod kurmak için Fabric profili oluştur.",
  "mods.installedToast": "{name} yüklendi",
  "mods.by": "{author} tarafından",

  "settings.title": "Ayarlar",
  "settings.username": "Kullanıcı adı",
  "settings.usernameHint": "Offline mod. 3-16 harf, rakam ya da alt çizgi.",
  "settings.usernameInvalid": "3-16 karakter: harf, rakam ya da _",
  "settings.memory": "Bellek (RAM)",
  "settings.memoryHint": "Çoğu mod paketi için 4-6 GB yeterli.",
  "settings.hide": "Oyun açıkken launcher'ı gizle",
  "settings.hideHint": "Oyun kapanınca geri gelir.",
  "settings.language": "Dil",
  "settings.saved": "Kaydedildi",
  "settings.about": "Vega açık kaynaklıdır, GPL-3.0 lisanslıdır.",

  "error.crashed": "Oyun beklenmedik şekilde kapandı (kod {code})",
  "window.minimize": "Küçült",
  "window.maximize": "Büyüt",
  "window.close": "Kapat",
};

const en: Record<keyof typeof tr, string> = {
  "nav.home": "Home",
  "nav.mods": "Mods",
  "nav.settings": "Settings",

  "home.welcome": "Welcome back",
  "home.play": "Play",
  "home.stop": "Stop",
  "home.running": "Game running",
  "home.mods": "Mods",
  "home.noMods": "No mods yet",
  "home.addMod": "Add",
  "home.vanillaNote": "Vanilla, no mods",
  "home.console": "Console",
  "home.firstRun": "Setting up your first profile…",

  "stage.metadata": "Version info",
  "stage.libraries": "Libraries",
  "stage.assets": "Sounds and textures",
  "stage.java": "Java",
  "stage.mods": "Mods",
  "stage.starting": "Starting",

  "profile.new": "New profile",
  "profile.openFolder": "Open folder",
  "profile.delete": "Delete profile",
  "profile.confirmDelete": "Sure? Worlds are deleted too",
  "profile.name": "Name",
  "profile.version": "Minecraft version",
  "profile.loader": "Mod loader",
  "profile.create": "Create",
  "profile.cancel": "Cancel",
  "profile.fabricHint": "Fabric lets you install mods. Vanilla is the plain game.",

  "mods.title": "Mods",
  "mods.search": "Search Modrinth",
  "mods.discover": "Discover",
  "mods.installed": "Installed",
  "mods.install": "Add",
  "mods.added": "Added",
  "mods.remove": "Remove",
  "mods.dependency": "dependency",
  "mods.empty": "No results",
  "mods.noneInstalled": "No mods in this profile yet. Add some from Discover.",
  "mods.vanillaTitle": "This profile has no mod loader",
  "mods.vanillaBody": "Create a Fabric profile to install mods.",
  "mods.installedToast": "{name} installed",
  "mods.by": "by {author}",

  "settings.title": "Settings",
  "settings.username": "Username",
  "settings.usernameHint": "Offline mode. 3-16 letters, digits or underscores.",
  "settings.usernameInvalid": "3-16 characters: letters, digits or _",
  "settings.memory": "Memory (RAM)",
  "settings.memoryHint": "4-6 GB is enough for most modpacks.",
  "settings.hide": "Hide the launcher while playing",
  "settings.hideHint": "It comes back when the game closes.",
  "settings.language": "Language",
  "settings.saved": "Saved",
  "settings.about": "Vega is open source under GPL-3.0.",

  "error.crashed": "The game closed unexpectedly (code {code})",
  "window.minimize": "Minimize",
  "window.maximize": "Maximize",
  "window.close": "Close",
};

const dictionaries: Record<Lang, Record<keyof typeof tr, string>> = { tr, en };

export type Key = keyof typeof tr;

export const locale = $state({ lang: (navigator.language.toLowerCase().startsWith("tr") ? "tr" : "en") as Lang });

export function t(key: Key, vars: Record<string, string | number> = {}): string {
  return dictionaries[locale.lang][key].replace(/\{(\w+)\}/g, (_, name: string) => String(vars[name] ?? ""));
}

export function compactNumber(n: number): string {
  return new Intl.NumberFormat(locale.lang, { notation: "compact", maximumFractionDigits: 1 }).format(n);
}
