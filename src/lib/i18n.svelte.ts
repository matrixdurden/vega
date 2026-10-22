export type Lang = "tr" | "en";

const tr = {
  "nav.profiles": "Profiller",
  "nav.settings": "Ayarlar",
  "nav.newProfile": "Yeni profil",

  "profile.play": "Oyna",
  "profile.stop": "Durdur",
  "profile.running": "Oyun çalışıyor",
  "profile.openFolder": "Klasörü aç",
  "profile.edit": "Düzenle",
  "profile.lastPlayed": "{when} oynandı",
  "profile.neverPlayed": "Henüz oynanmadı",
  "profile.console": "Konsol",
  "profile.empty": "İlk profilini oluştur",
  "profile.emptyBody": "Her profilin kendi sürümü, modları ve dünyaları olur.",
  "profile.firstRun": "İlk profil hazırlanıyor…",

  "tab.mods": "Modlar",
  "tab.worlds": "Dünyalar",
  "tab.settings": "Ayarlar",

  "stage.metadata": "Sürüm bilgisi",
  "stage.libraries": "Kütüphaneler",
  "stage.assets": "Ses ve dokular",
  "stage.java": "Java",
  "stage.mods": "Modlar",
  "stage.starting": "Başlatılıyor",

  "dialog.create": "Yeni profil",
  "dialog.edit": "Profili düzenle",
  "dialog.icon": "İkon",
  "dialog.name": "İsim",
  "dialog.version": "Minecraft sürümü",
  "dialog.loader": "Mod yükleyici",
  "dialog.loaderHint": "Fabric ile mod kurabilirsin. Vanilla saf oyundur.",
  "dialog.modsWarning": "Sürüm değişince yüklü modlar uyumsuz kalabilir.",
  "dialog.save": "Kaydet",
  "dialog.createButton": "Oluştur",
  "dialog.cancel": "Vazgeç",

  "mods.add": "Mod ekle",
  "mods.search": "Modrinth'te ara",
  "mods.install": "Ekle",
  "mods.added": "Yüklü",
  "mods.remove": "Kaldır",
  "mods.enable": "Modu aç ya da kapat",
  "mods.dependency": "bağımlılık",
  "mods.empty": "Sonuç bulunamadı",
  "mods.none": "Bu profilde henüz mod yok",
  "mods.noneBody": "Modrinth'ten Fabric modları ekleyebilirsin. Gerekli kütüphaneler otomatik gelir.",
  "mods.vanillaTitle": "Bu profil modsuz",
  "mods.vanillaBody": "Mod kurmak için profili düzenleyip Fabric'e geçir.",
  "mods.installedToast": "{name} yüklendi",
  "mods.by": "{author} tarafından",
  "mods.browserTitle": "Mod ekle",
  "mods.for": "{profile} · Minecraft {version}",

  "worlds.none": "Henüz dünya yok",
  "worlds.noneBody": "Oyna'ya bas ve oyunda yeni bir dünya oluştur.",
  "worlds.openFolder": "Kayıt klasörü",

  "psettings.memory": "Bellek",
  "psettings.memoryDefault": "Genel ayarı kullan ({gb} GB)",
  "psettings.memoryHint": "Büyük mod paketleri için bu profile özel bellek ver.",
  "psettings.folders": "Klasörler",
  "psettings.danger": "Profili sil",
  "psettings.dangerHint": "Modlar, ayarlar ve dünyalar kalıcı olarak silinir.",
  "psettings.delete": "Profili sil",
  "psettings.confirmDelete": "Emin misin? Tekrar bas",

  "settings.title": "Ayarlar",
  "settings.username": "Kullanıcı adı",
  "settings.usernameHint": "Offline mod. 3-16 harf, rakam ya da alt çizgi.",
  "settings.usernameInvalid": "3-16 karakter: harf, rakam ya da _",
  "settings.memory": "Varsayılan bellek",
  "settings.memoryHint": "Profiller kendi değerini seçmezse bu kullanılır.",
  "settings.hide": "Oyun açıkken launcher'ı gizle",
  "settings.hideHint": "Oyun kapanınca geri gelir.",
  "settings.language": "Dil",
  "settings.about": "Vega açık kaynaklıdır, GPL-3.0 lisanslıdır.",

  "error.crashed": "Oyun beklenmedik şekilde kapandı (kod {code})",
  "window.minimize": "Küçült",
  "window.maximize": "Büyüt",
  "window.close": "Kapat",
};

const en: Record<keyof typeof tr, string> = {
  "nav.profiles": "Profiles",
  "nav.settings": "Settings",
  "nav.newProfile": "New profile",

  "profile.play": "Play",
  "profile.stop": "Stop",
  "profile.running": "Game running",
  "profile.openFolder": "Open folder",
  "profile.edit": "Edit",
  "profile.lastPlayed": "Played {when}",
  "profile.neverPlayed": "Not played yet",
  "profile.console": "Console",
  "profile.empty": "Create your first profile",
  "profile.emptyBody": "Each profile has its own version, mods and worlds.",
  "profile.firstRun": "Setting up your first profile…",

  "tab.mods": "Mods",
  "tab.worlds": "Worlds",
  "tab.settings": "Settings",

  "stage.metadata": "Version info",
  "stage.libraries": "Libraries",
  "stage.assets": "Sounds and textures",
  "stage.java": "Java",
  "stage.mods": "Mods",
  "stage.starting": "Starting",

  "dialog.create": "New profile",
  "dialog.edit": "Edit profile",
  "dialog.icon": "Icon",
  "dialog.name": "Name",
  "dialog.version": "Minecraft version",
  "dialog.loader": "Mod loader",
  "dialog.loaderHint": "Fabric lets you install mods. Vanilla is the plain game.",
  "dialog.modsWarning": "Installed mods may not work after changing the version.",
  "dialog.save": "Save",
  "dialog.createButton": "Create",
  "dialog.cancel": "Cancel",

  "mods.add": "Add mods",
  "mods.search": "Search Modrinth",
  "mods.install": "Add",
  "mods.added": "Added",
  "mods.remove": "Remove",
  "mods.enable": "Turn mod on or off",
  "mods.dependency": "dependency",
  "mods.empty": "No results",
  "mods.none": "No mods in this profile yet",
  "mods.noneBody": "Add Fabric mods from Modrinth. Required libraries come along automatically.",
  "mods.vanillaTitle": "This profile has no mod loader",
  "mods.vanillaBody": "Edit the profile and switch it to Fabric to install mods.",
  "mods.installedToast": "{name} installed",
  "mods.by": "by {author}",
  "mods.browserTitle": "Add mods",
  "mods.for": "{profile} · Minecraft {version}",

  "worlds.none": "No worlds yet",
  "worlds.noneBody": "Press Play and create a world in the game.",
  "worlds.openFolder": "Saves folder",

  "psettings.memory": "Memory",
  "psettings.memoryDefault": "Use the global setting ({gb} GB)",
  "psettings.memoryHint": "Give big modpacks their own memory.",
  "psettings.folders": "Folders",
  "psettings.danger": "Delete profile",
  "psettings.dangerHint": "Mods, settings and worlds are deleted for good.",
  "psettings.delete": "Delete profile",
  "psettings.confirmDelete": "Sure? Press again",

  "settings.title": "Settings",
  "settings.username": "Username",
  "settings.usernameHint": "Offline mode. 3-16 letters, digits or underscores.",
  "settings.usernameInvalid": "3-16 characters: letters, digits or _",
  "settings.memory": "Default memory",
  "settings.memoryHint": "Used by profiles that do not set their own.",
  "settings.hide": "Hide the launcher while playing",
  "settings.hideHint": "It comes back when the game closes.",
  "settings.language": "Language",
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

/** "2 hours ago", "yesterday" and so on, from unix seconds. */
export function relativeTime(unixSeconds: number): string {
  const seconds = unixSeconds - Date.now() / 1000;
  const format = new Intl.RelativeTimeFormat(locale.lang, { numeric: "auto" });
  const units: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["week", 604_800],
    ["day", 86_400],
    ["hour", 3600],
    ["minute", 60],
  ];
  for (const [unit, size] of units) {
    if (Math.abs(seconds) >= size) return format.format(Math.round(seconds / size), unit);
  }
  return format.format(0, "minute");
}
