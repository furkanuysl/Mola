import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, ReminderAction, ReminderView } from "./api";
import { useT } from "./i18n";

function formatClock(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function Reminder() {
  const t = useT();
  const [view, setView] = useState<ReminderView | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.getReminder().then(setView);
    const unlisten = listen<ReminderView | null>("reminder", (e) => {
      setView(e.payload);
      setError(null);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  if (!view) return null;

  const act = (action: ReminderAction) => api.reminderAction(action).catch((e) => setError(String(e)));

  const snoozeButton = (
    <button className="secondary" disabled={view.snoozesLeft === 0} onClick={() => act("snooze")}>
      {t.snooze(view.snoozeMin)} <small>{t.snoozesLeft(view.snoozesLeft)}</small>
    </button>
  );
  const snoozeNote = view.snoozesLeft === 0 && <p className="note">{t.noSnoozesLeft}</p>;

  let body;
  if (view.kind === "water") {
    const pct = Math.min(100, (view.waterToday / view.waterGoal) * 100);
    body = (
      <>
        <Head icon="💧" title={t.waterTitle} />
        <p>{t.waterToday(view.waterToday, view.waterGoal)}</p>
        <div className="bar">
          <span style={{ width: `${pct}%` }} />
        </div>
        <div className="actions">
          <button className="primary" onClick={() => act("drink")}>
            {t.drank(view.glassMl)}
          </button>
          {snoozeButton}
          <button className="ghost" onClick={() => act("skip")}>
            {t.skip}
          </button>
        </div>
        {snoozeNote}
      </>
    );
  } else if (view.phase === "prompt") {
    body = (
      <>
        <Head icon="🚶" title={t.moveTitle} />
        <p>{t.moveBody(view.sittingMin, Math.round(view.breakTotal / 60))}</p>
        <div className="actions">
          <button className="primary" onClick={() => act("start_break")}>
            {t.startBreak}
          </button>
          {snoozeButton}
        </div>
        {snoozeNote}
      </>
    );
  } else if (view.phase === "break") {
    const elapsed = view.breakTotal - view.breakRemaining;
    const tip = t.tips[Math.floor(elapsed / 30) % t.tips.length];
    body = (
      <>
        <Head icon="⏱️" title={t.breakTitle} />
        <div className="clock">{formatClock(view.breakRemaining)}</div>
        <div className="bar">
          <span style={{ width: `${(elapsed / view.breakTotal) * 100}%` }} />
        </div>
        <p className="tip">{tip}</p>
      </>
    );
  } else {
    body = (
      <>
        <Head icon="✅" title={t.doneTitle} />
        <p>{t.doneBody}</p>
        <div className="actions">
          <button className="primary" onClick={() => act("dismiss")}>
            {t.close}
          </button>
        </div>
      </>
    );
  }

  return (
    <div className={`reminder ${view.kind}`}>
      {body}
      {error && <p className="note error">{t.errors[error] ?? error}</p>}
    </div>
  );
}

function Head({ icon, title }: { icon: string; title: string }) {
  return (
    <header className="reminder-head" data-tauri-drag-region>
      <span className="badge" data-tauri-drag-region>
        {icon}
      </span>
      <h1 data-tauri-drag-region>{title}</h1>
    </header>
  );
}
