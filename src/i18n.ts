import { createContext, useContext } from "react";

export type Lang = "tr" | "en";

const tr = {
  appName: "Mola",
  // Hatırlatma penceresi
  waterTitle: "Su içme vakti",
  waterToday: (ml: number, goal: number) => `Bugün ${ml} / ${goal} ml`,
  drank: (ml: number) => `İçtim (+${ml} ml)`,
  skip: "Atla",
  snooze: (min: number) => `${min} dk ertele`,
  snoozesLeft: (n: number) => `(${n} hak)`,
  noSnoozesLeft: "Erteleme hakkın bitti.",
  moveTitle: "Kalkma vakti",
  moveBody: (sitting: number, brk: number) => `${sitting} dakikadır oturuyorsun. ${brk} dakika kalkıp yürü, biraz esne.`,
  startBreak: "Molaya başla",
  breakTitle: "Mola",
  doneTitle: "Mola tamam",
  doneBody: "Harika! Oturma sayacı sıfırlandı.",
  close: "Kapat",
  tips: [
    "Kalk ve birkaç dakika yürü.",
    "Omuzlarını geriye doğru 10 kez çevir.",
    "Başını yavaşça iki yana eğerek boynunu esnet.",
    "Pencereden uzağa bak, gözlerini dinlendir.",
    "Parmak uçlarında yükselip in, 15 kez.",
    "Ellerini birleştirip yukarı uzan, 10 saniye tut.",
  ],
  errors: {
    nothing_to_skip: "Atlanacak su hatırlatması yok.",
    nothing_to_snooze: "Ertelenecek hatırlatma yok.",
    no_snoozes_left: "Erteleme hakkın bitti.",
    nothing_to_start: "Başlatılacak mola yok.",
    unknown_action: "Bilinmeyen işlem.",
  } as Record<string, string>,

  // Panel
  pauseHour: "1 saat duraklat",
  resume: "Devam et",
  minutesLeft: (n: number) => `(${n} dk kaldı)`,
  outsideHours: "Mesai saatleri dışındasın, hatırlatmalar beklemede.",
  water: "Su",
  move: "Hareket",
  sittingFor: "dk oturuyorsun",
  nextWater: (n: number) => `Sonraki hatırlatma: ${n} dk`,
  nextBreak: (n: number) => `Sonraki mola: ${n} dk`,
  off: "Kapalı",
  iDrank: "Su içtim",
  takeBreakNow: "Şimdi mola ver",
  todaySummary: (breaks: number, snoozes: number) => `Bugün ${breaks} mola · ${snoozes} erteleme`,
  last7Days: "Son 7 gün",
  dayTooltip: (date: string, ml: number, breaks: number) => `${date}: ${ml} ml, ${breaks} mola`,

  // Ayarlar
  settings: "Ayarlar",
  waterEnabled: "Su hatırlatıcısı açık",
  waterInterval: "Hatırlatma aralığı",
  waterGoal: "Günlük hedef",
  glass: "Bir bardak",
  moveEnabled: "Hareket hatırlatıcısı açık",
  moveInterval: "Kesintisiz oturma süresi",
  breakLength: "Mola süresi",
  idleReset: "Bu kadar boşta kalınca kalktın say",
  snoozeSection: "Erteleme",
  snoozeLength: "Erteleme süresi",
  maxSnoozes: "En fazla erteleme",
  workHours: "Mesai saatleri",
  workHoursEnabled: "Sadece mesai saatlerinde hatırlat",
  startEnd: "Başlangıç / bitiş",
  weekdaysOnly: "Sadece hafta içi",
  general: "Genel",
  autostart: "Windows açılınca başlat",
  language: "Dil",
  languageAuto: "Sistem dili",
  save: "Kaydet",
  saved: "Kaydedildi ✓",
  unitMin: "dk",
  unitMl: "ml",
  unitTimes: "kez",
};

export type Dict = typeof tr;

const en: Dict = {
  appName: "Mola",
  waterTitle: "Time to drink water",
  waterToday: (ml, goal) => `Today ${ml} / ${goal} ml`,
  drank: (ml) => `I drank (+${ml} ml)`,
  skip: "Skip",
  snooze: (min) => `Snooze ${min} min`,
  snoozesLeft: (n) => `(${n} left)`,
  noSnoozesLeft: "No snoozes left.",
  moveTitle: "Time to stand up",
  moveBody: (sitting, brk) => `You've been sitting for ${sitting} minutes. Get up, walk and stretch for ${brk} minutes.`,
  startBreak: "Start break",
  breakTitle: "Break",
  doneTitle: "Break done",
  doneBody: "Nice! Your sitting timer has been reset.",
  close: "Close",
  tips: [
    "Stand up and walk for a few minutes.",
    "Roll your shoulders backwards 10 times.",
    "Gently tilt your head side to side to stretch your neck.",
    "Look out the window and rest your eyes.",
    "Rise onto your toes and back down, 15 times.",
    "Clasp your hands and reach up, hold for 10 seconds.",
  ],
  errors: {
    nothing_to_skip: "There is no water reminder to skip.",
    nothing_to_snooze: "There is no reminder to snooze.",
    no_snoozes_left: "No snoozes left.",
    nothing_to_start: "There is no break to start.",
    unknown_action: "Unknown action.",
  },

  pauseHour: "Pause for 1 hour",
  resume: "Resume",
  minutesLeft: (n) => `(${n} min left)`,
  outsideHours: "Outside working hours, reminders are on hold.",
  water: "Water",
  move: "Movement",
  sittingFor: "min sitting",
  nextWater: (n) => `Next reminder: ${n} min`,
  nextBreak: (n) => `Next break: ${n} min`,
  off: "Off",
  iDrank: "I drank water",
  takeBreakNow: "Take a break now",
  todaySummary: (breaks, snoozes) => `Today ${breaks} breaks · ${snoozes} snoozes`,
  last7Days: "Last 7 days",
  dayTooltip: (date, ml, breaks) => `${date}: ${ml} ml, ${breaks} breaks`,

  settings: "Settings",
  waterEnabled: "Water reminder on",
  waterInterval: "Reminder interval",
  waterGoal: "Daily goal",
  glass: "One glass",
  moveEnabled: "Movement reminder on",
  moveInterval: "Continuous sitting time",
  breakLength: "Break length",
  idleReset: "Count as stood up after being idle for",
  snoozeSection: "Snooze",
  snoozeLength: "Snooze length",
  maxSnoozes: "Maximum snoozes",
  workHours: "Working hours",
  workHoursEnabled: "Only remind during working hours",
  startEnd: "Start / end",
  weekdaysOnly: "Weekdays only",
  general: "General",
  autostart: "Start with Windows",
  language: "Language",
  languageAuto: "System language",
  save: "Save",
  saved: "Saved ✓",
  unitMin: "min",
  unitMl: "ml",
  unitTimes: "×",
};

export const dictionaries: Record<Lang, Dict> = { tr, en };

export const LangContext = createContext<Lang>("tr");

export function useLang() {
  return useContext(LangContext);
}

export function useT() {
  return dictionaries[useLang()];
}
