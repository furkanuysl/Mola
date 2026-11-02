import { useState } from "react";
import { api, Status, Technique } from "./api";
import { useT } from "./i18n";

const TECHNIQUES: Technique[] = ["pomodoro", "52_17", "90_20", "flowtime"];

export function formatClock(secs: number) {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = (secs % 60).toString().padStart(2, "0");
  return h > 0 ? `${h}:${m.toString().padStart(2, "0")}:${s}` : `${m}:${s}`;
}

/** Odak zamanlayıcısı kartı: teknik seçimi, geri sayım ve tur bilgisi. */
export function FocusCard({ status }: { status: Status }) {
  const t = useT();
  const f = status.focus;
  const [picked, setPicked] = useState<Technique>(f.technique);
  const technique = f.phase === "off" ? picked : f.technique;
  const info = t.techniques[technique];

  let body;
  if (f.phase === "focus") {
    const shown = f.target !== null ? Math.max(0, f.target - f.elapsed) : f.elapsed;
    const pct = f.target !== null ? (f.elapsed / f.target) * 100 : 0;
    body = (
      <>
        <p className="muted">
          {info.name}
          {f.rounds !== null && ` · ${t.round(f.round, f.rounds)}`}
          {f.paused && ` · ${t.focusPaused}`}
        </p>
        <div className={`clock ${f.paused ? "paused" : ""}`}>{formatClock(shown)}</div>
        {f.target !== null ? (
          <div className="bar">
            <span style={{ width: `${Math.min(100, pct)}%` }} />
          </div>
        ) : (
          <p className="muted center">{t.flowtimeHint}</p>
        )}
        <div className="actions">
          <button className="primary" onClick={() => api.focusPause(!f.paused)}>
            {f.paused ? t.continueExercise : t.pauseExercise}
          </button>
          <button className="secondary" onClick={api.focusBreakNow}>
            {t.breakNow}
          </button>
          <button className="ghost" onClick={api.focusStop}>
            {t.stop}
          </button>
        </div>
      </>
    );
  } else if (f.phase === "break") {
    body = (
      <>
        <p className="muted">
          {info.name} · {t.onBreak}
        </p>
        <div className="clock">{formatClock(f.breakRemaining ?? 0)}</div>
        <p className="muted center">{t.focusBreakHint}</p>
        <div className="actions">
          <button className="ghost" onClick={api.focusStop}>
            {t.stop}
          </button>
        </div>
      </>
    );
  } else {
    const ready = f.phase === "ready";
    body = (
      <>
        {!ready && (
          <div className="chips" role="radiogroup">
            {TECHNIQUES.map((id) => (
              <button
                key={id}
                role="radio"
                aria-checked={picked === id}
                className={`chip ${picked === id ? "on" : ""}`}
                onClick={() => setPicked(id)}
              >
                {t.techniques[id].name}
              </button>
            ))}
          </div>
        )}
        <p className="muted">{ready ? t.readyForNext(f.round, f.rounds) : info.desc}</p>
        <div className="actions">
          <button className="primary" onClick={() => api.focusStart(technique)}>
            {ready ? t.startNextRound : t.startFocus}
          </button>
          {ready && (
            <button className="ghost" onClick={api.focusStop}>
              {t.stop}
            </button>
          )}
        </div>
      </>
    );
  }

  return (
    <section className="panel focus-card">
      <div className="panel-head">
        <h2>🎯 {t.focus}</h2>
        <span className="muted">{t.focusToday(status.focusMin)}</span>
      </div>
      {body}
    </section>
  );
}
