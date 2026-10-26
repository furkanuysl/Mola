import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./api";
import { Dashboard } from "./Dashboard";
import { Lang, LangContext } from "./i18n";
import { Reminder } from "./Reminder";
import "./App.css";

const isReminder = getCurrentWindow().label === "reminder";
document.documentElement.classList.toggle("reminder-window", isReminder);

export default function App() {
  const [lang, setLang] = useState<Lang | null>(null);

  useEffect(() => {
    api.getLanguage().then(setLang);
    const unlisten = listen<Lang>("language", (e) => setLang(e.payload));
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (lang) document.documentElement.lang = lang;
  }, [lang]);

  if (!lang) return null;
  return <LangContext.Provider value={lang}>{isReminder ? <Reminder /> : <Dashboard />}</LangContext.Provider>;
}
