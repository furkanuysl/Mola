import type { Lang } from "./i18n";

/** Masa başı çalışanlar için duruş, boyun, sırt ve göz egzersizleri.
 *  Boyun fıtığında riskli olan hareketler (boyun çevirme, zorlamalı esnetme) bilerek yok. */

interface Text {
  name: string;
  /** Hatırlatma penceresinde gösterilen tek satırlık özet. */
  short: string;
  steps: string[];
}

export interface Exercise {
  id: string;
  icon: string;
  secs: number;
  tr: Text;
  en: Text;
}

export const EXERCISES: Record<string, Exercise> = {
  chinTuck: {
    id: "chinTuck",
    icon: "🙂",
    secs: 60,
    tr: {
      name: "Çene geri çekme (chin tuck)",
      short: "Çeneni düz bir çizgide geriye çek, 5 sn tut · 10 tekrar",
      steps: [
        "Dik otur, karşıya bak, omuzların rahat olsun.",
        "Başını eğmeden çeneni düz bir çizgide geriye çek (çift çene yapar gibi).",
        "Ensende hafif bir gerilme hisset, 5 saniye tut ve bırak.",
        "10 tekrar yap.",
      ],
    },
    en: {
      name: "Chin tuck",
      short: "Pull your chin straight back, hold 5 s · 10 reps",
      steps: [
        "Sit tall, look straight ahead, keep your shoulders relaxed.",
        "Without tilting your head, glide your chin straight back (make a double chin).",
        "Feel a light stretch at the back of your neck, hold for 5 seconds and release.",
        "Do 10 reps.",
      ],
    },
  },
  scapSqueeze: {
    id: "scapSqueeze",
    icon: "🫸",
    secs: 50,
    tr: {
      name: "Kürek kemiği sıkıştırma",
      short: "Kürek kemiklerini geriye ve aşağı sık, 5 sn tut · 10 tekrar",
      steps: [
        "Kolların yanda, omuzların kulaklarından uzakta olsun.",
        "Kürek kemiklerini geriye ve aşağıya doğru birbirine yaklaştır.",
        "5 saniye tut ve bırak. Omuzlarını kaldırma.",
        "10 tekrar yap.",
      ],
    },
    en: {
      name: "Shoulder blade squeeze",
      short: "Squeeze shoulder blades back and down, hold 5 s · 10 reps",
      steps: [
        "Arms by your sides, shoulders away from your ears.",
        "Draw your shoulder blades back and down towards each other.",
        "Hold for 5 seconds and release. Don't shrug.",
        "Do 10 reps.",
      ],
    },
  },
  wSqueeze: {
    id: "wSqueeze",
    icon: "🙆",
    secs: 50,
    tr: {
      name: "W egzersizi",
      short: "Kollarla W yap, dirsekleri geri-aşağı çek · 10 tekrar",
      steps: [
        "Dirseklerini bük, avuçların öne bakacak şekilde kollarınla W harfi yap.",
        "Dirseklerini geriye ve aşağı çekerek kürek kemiklerini sık.",
        "3 saniye tut, gevşe.",
        "10 tekrar yap.",
      ],
    },
    en: {
      name: "W squeeze",
      short: "Make a W with your arms, pull elbows back and down · 10 reps",
      steps: [
        "Bend your elbows and make a W with your arms, palms facing forward.",
        "Pull your elbows back and down, squeezing your shoulder blades.",
        "Hold for 3 seconds, relax.",
        "Do 10 reps.",
      ],
    },
  },
  chestOpener: {
    id: "chestOpener",
    icon: "🚪",
    secs: 40,
    tr: {
      name: "Göğüs açma (kapı esnetmesi)",
      short: "Kapı kasasında göğsünü aç, 20–30 sn tut",
      steps: [
        "Ön kollarını omuz hizasında kapı kasasının iki yanına yasla.",
        "Bir adım öne çık, göğsünün önünde gerilme hisset.",
        "20–30 saniye tut, nefesini tutma.",
        "Kapı yoksa ellerini arkada birleştirip göğsünü aç.",
      ],
    },
    en: {
      name: "Doorway chest stretch",
      short: "Open your chest in a doorway, hold 20–30 s",
      steps: [
        "Place your forearms on both sides of a doorframe at shoulder height.",
        "Step forward until you feel a stretch across your chest.",
        "Hold for 20–30 seconds and keep breathing.",
        "No doorway? Clasp your hands behind you and open your chest.",
      ],
    },
  },
  thoracicExt: {
    id: "thoracicExt",
    icon: "🪑",
    secs: 45,
    tr: {
      name: "Sandalyede sırt açma",
      short: "Üst sırtını sandalye arkalığından geriye aç · 5 tekrar",
      steps: [
        "Sırtı sabit bir sandalyede otur, ellerini başının arkasında birleştir.",
        "Üst sırtını sandalye arkalığının üzerinden yavaşça geriye doğru aç.",
        "Hareketi boynunla değil göğsünle yap; başın ellerine yaslı kalsın.",
        "3 saniye tut, 5 tekrar yap.",
      ],
    },
    en: {
      name: "Seated upper-back extension",
      short: "Arch your upper back over the chair back · 5 reps",
      steps: [
        "Sit on a stable chair and clasp your hands behind your head.",
        "Slowly arch your upper back over the backrest.",
        "Lead with your chest, not your neck; keep your head resting in your hands.",
        "Hold for 3 seconds, do 5 reps.",
      ],
    },
  },
  shoulderRolls: {
    id: "shoulderRolls",
    icon: "🔄",
    secs: 30,
    tr: {
      name: "Omuz çevirme",
      short: "Omuzlarını geriye doğru büyük daireler çizerek çevir · 10 kez",
      steps: [
        "Omuzlarını kulaklarına doğru kaldır.",
        "Geriye ve aşağıya doğru büyük, yavaş daireler çiz.",
        "10 kez geriye çevir.",
      ],
    },
    en: {
      name: "Shoulder rolls",
      short: "Roll your shoulders back in big circles · 10 times",
      steps: [
        "Lift your shoulders towards your ears.",
        "Roll them back and down in big, slow circles.",
        "Do 10 backward rolls.",
      ],
    },
  },
  upperTrap: {
    id: "upperTrap",
    icon: "↔️",
    secs: 50,
    tr: {
      name: "Boyun yan esnetme",
      short: "Kulağını omzuna doğru yavaşça eğ, her yana 20 sn",
      steps: [
        "Sağ elinle sandalyenin altını tutarak sağ omzunu aşağıda sabitle.",
        "Sol kulağını sol omzuna doğru yavaşça eğ.",
        "20 saniye tut; yaylanma, zorlama.",
        "Diğer tarafa geç.",
      ],
    },
    en: {
      name: "Side neck stretch",
      short: "Gently tilt your ear towards your shoulder, 20 s each side",
      steps: [
        "Hold the edge of your seat with your right hand to keep the shoulder down.",
        "Slowly tilt your left ear towards your left shoulder.",
        "Hold for 20 seconds; don't bounce or force it.",
        "Switch sides.",
      ],
    },
  },
  levator: {
    id: "levator",
    icon: "↘️",
    secs: 50,
    tr: {
      name: "Boyun arka esnetme",
      short: "Başını hafif çevirip burnunu koltuk altına indir, her yana 20 sn",
      steps: [
        "Başını yaklaşık 45° sağa çevir.",
        "Burnunu sağ koltuk altına doğru yavaşça indir.",
        "İstersen elinle çok hafif yardım et; 20 saniye tut.",
        "Diğer tarafa geç.",
      ],
    },
    en: {
      name: "Back-of-neck stretch",
      short: "Turn slightly and lower your nose towards your armpit, 20 s each side",
      steps: [
        "Turn your head about 45° to the right.",
        "Slowly lower your nose towards your right armpit.",
        "Assist very lightly with your hand if you like; hold for 20 seconds.",
        "Switch sides.",
      ],
    },
  },
  catCow: {
    id: "catCow",
    icon: "🐈",
    secs: 45,
    tr: {
      name: "Oturarak kedi-deve",
      short: "Nefes alırken göğsü aç, verirken sırtı yuvarla · 8 tekrar",
      steps: [
        "Sandalyenin ön kısmında otur, ellerini dizlerine koy.",
        "Nefes alırken göğsünü öne aç, belini hafifçe çukurlaştır.",
        "Nefes verirken sırtını yuvarla, çeneni hafifçe içeri al.",
        "Yavaşça 8 tekrar yap.",
      ],
    },
    en: {
      name: "Seated cat-cow",
      short: "Open your chest as you inhale, round your back as you exhale · 8 reps",
      steps: [
        "Sit near the front of your chair, hands on your knees.",
        "Inhale: open your chest forward and gently arch your lower back.",
        "Exhale: round your back and gently tuck your chin.",
        "Do 8 slow reps.",
      ],
    },
  },
  backExt: {
    id: "backExt",
    icon: "🧍",
    secs: 30,
    tr: {
      name: "Ayakta bel açma",
      short: "Ellerin belinde, yavaşça geriye esne · 5 tekrar",
      steps: [
        "Ayağa kalk, ellerini belinin arkasına koy.",
        "Kalçanı hafifçe öne iterek yavaşça geriye doğru esne.",
        "2 saniye tut, 5 tekrar yap.",
      ],
    },
    en: {
      name: "Standing back bend",
      short: "Hands on your lower back, gently lean back · 5 reps",
      steps: [
        "Stand up and place your hands on your lower back.",
        "Push your hips slightly forward and gently lean back.",
        "Hold for 2 seconds, do 5 reps.",
      ],
    },
  },
  eyes: {
    id: "eyes",
    icon: "👀",
    secs: 20,
    tr: {
      name: "20-20-20 göz molası",
      short: "6 metre uzağa 20 saniye bak, gözlerini kırp",
      steps: [
        "Gözlerini ekrandan ayır.",
        "En az 6 metre uzaktaki bir noktaya bak.",
        "20 saniye boyunca odağını orada tut, sık sık göz kırp.",
      ],
    },
    en: {
      name: "20-20-20 eye break",
      short: "Look 6 metres away for 20 seconds, blink",
      steps: [
        "Take your eyes off the screen.",
        "Look at something at least 6 metres (20 feet) away.",
        "Keep your focus there for 20 seconds and blink often.",
      ],
    },
  },
  walk: {
    id: "walk",
    icon: "🚶",
    secs: 60,
    tr: {
      name: "Kısa yürüyüş",
      short: "Kalk, birkaç adım yürü, su al",
      steps: ["Ayağa kalk ve biraz yürü.", "Bir bardak su al ya da pencereye git.", "Omuzların rahat, adımların sakin olsun."],
    },
    en: {
      name: "Short walk",
      short: "Get up, take a few steps, grab some water",
      steps: ["Stand up and walk around.", "Grab a glass of water or go to a window.", "Keep your shoulders relaxed and your pace easy."],
    },
  },
};

export type RoutineId = "mixed" | "posture" | "neck" | "back" | "quick";

export interface Routine {
  id: RoutineId;
  icon: string;
  exercises: string[];
  tr: { name: string; desc: string };
  en: { name: string; desc: string };
}

export const ROUTINES: Routine[] = [
  {
    id: "posture",
    icon: "🧍‍♂️",
    exercises: ["chinTuck", "scapSqueeze", "wSqueeze", "chestOpener", "thoracicExt"],
    tr: { name: "Kamburluk karşıtı", desc: "Öne düşen baş ve yuvarlanan omuzlar için duruşu düzelten hareketler." },
    en: { name: "Posture reset", desc: "Counters forward head and rounded shoulders." },
  },
  {
    id: "neck",
    icon: "🧣",
    exercises: ["chinTuck", "shoulderRolls", "upperTrap", "levator", "scapSqueeze"],
    tr: { name: "Boyun rahatlatma", desc: "Boyun ve omuz gerginliğini azaltan nazik esnetmeler." },
    en: { name: "Neck relief", desc: "Gentle stretches that ease neck and shoulder tension." },
  },
  {
    id: "back",
    icon: "🪑",
    exercises: ["catCow", "thoracicExt", "backExt", "walk"],
    tr: { name: "Bel ve sırt", desc: "Uzun oturmanın sırtta yarattığı sertliği açar." },
    en: { name: "Back and spine", desc: "Loosens the stiffness long sitting leaves in your back." },
  },
  {
    id: "quick",
    icon: "⚡",
    exercises: ["chinTuck", "shoulderRolls", "eyes"],
    tr: { name: "Hızlı mola", desc: "Vaktin azsa: boyun, omuz ve göz için 2 dakika." },
    en: { name: "Quick break", desc: "Short on time: 2 minutes for neck, shoulders and eyes." },
  },
  {
    id: "mixed",
    icon: "🔀",
    exercises: ["walk", "chinTuck", "scapSqueeze", "upperTrap", "chestOpener", "eyes", "catCow"],
    tr: { name: "Karışık", desc: "Yürüyüş, duruş, boyun ve göz hareketlerinden bir karışım." },
    en: { name: "Mixed", desc: "A mix of walking, posture, neck and eye exercises." },
  },
];

export function routine(id: string): Routine {
  return ROUTINES.find((r) => r.id === id) ?? ROUTINES[ROUTINES.length - 1];
}

export function routineSecs(r: Routine): number {
  return r.exercises.reduce((sum, id) => sum + EXERCISES[id].secs, 0);
}

/** Molanın `elapsed`. saniyesinde rutindeki hangi hareketin yapıldığını döndürür; rutin bitince başa sarar. */
export function exerciseAt(r: Routine, elapsed: number): { exercise: Exercise; left: number } {
  let t = elapsed % routineSecs(r);
  for (const id of r.exercises) {
    const ex = EXERCISES[id];
    if (t < ex.secs) return { exercise: ex, left: ex.secs - t };
    t -= ex.secs;
  }
  const ex = EXERCISES[r.exercises[0]];
  return { exercise: ex, left: ex.secs };
}

export function text<T extends { tr: unknown; en: unknown }>(item: T, lang: Lang): T["tr"] {
  return item[lang] as T["tr"];
}

export interface Breath {
  id: string;
  icon: string;
  /** Saniye: nefes al, tut, ver, boşken tut (0 = yok). */
  pattern: [number, number, number, number];
  cycles: number;
  tr: { name: string; desc: string };
  en: { name: string; desc: string };
}

export const BREATHS: Breath[] = [
  {
    id: "box",
    icon: "🟦",
    pattern: [4, 4, 4, 4],
    cycles: 4,
    tr: { name: "Kutu nefesi", desc: "4 sn al, 4 sn tut, 4 sn ver, 4 sn bekle. Odağı toplar, gerginliği alır." },
    en: { name: "Box breathing", desc: "Inhale 4 s, hold 4 s, exhale 4 s, wait 4 s. Calms and refocuses." },
  },
  {
    id: "relax",
    icon: "🌙",
    pattern: [4, 7, 8, 0],
    cycles: 4,
    tr: { name: "4-7-8 rahatlama", desc: "4 sn al, 7 sn tut, 8 sn yavaşça ver. Stresli anlar için." },
    en: { name: "4-7-8 relax", desc: "Inhale 4 s, hold 7 s, exhale slowly for 8 s. For stressful moments." },
  },
];
