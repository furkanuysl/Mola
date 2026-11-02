import { useState } from "react";
import type { HistoryDay, Status } from "./api";
import { useLang, useT } from "./i18n";

/** Arka uçtaki `GOOD_DAY_SCORE` ile aynı. */
export const GOOD_DAY_SCORE = 70;

/** Günün skoru ve seri. */
export function ScoreRow({ status }: { status: Status }) {
  const t = useT();
  const good = status.score >= GOOD_DAY_SCORE;
  return (
    <section className="score-row">
      <article className="panel score">
        <div className="ring" style={{ "--pct": `${status.score}%` } as React.CSSProperties}>
          <span>{status.score}</span>
        </div>
        <div>
          <h2>{t.todayScore}</h2>
          <p className="muted">{good ? t.goodDay : t.scoreHint(GOOD_DAY_SCORE)}</p>
        </div>
      </article>
      <article className="panel streak">
        <p className="streak-count">🔥 {status.streak}</p>
        <div>
          <h2>{t.streak(status.streak)}</h2>
          <p className="muted">{status.streak > 0 ? t.streakKeep : t.streakStart}</p>
        </div>
      </article>
    </section>
  );
}

/** Son 7 günün paylaşılabilir özeti. */
export function WeekSummary({ history, streak }: { history: HistoryDay[]; streak: number }) {
  const t = useT();
  const lang = useLang();
  const [copied, setCopied] = useState(false);
  if (history.length === 0) return null;

  const sum = (f: (d: HistoryDay) => number) => history.reduce((a, d) => a + f(d), 0);
  const focusH = sum((d) => d.focusSecs) / 3600;
  const waterL = sum((d) => d.waterMl) / 1000;
  const breaks = sum((d) => d.breaks);
  const goodDays = history.filter((d) => d.score >= GOOD_DAY_SCORE).length;
  const best = history.reduce((a, d) => (d.score > a.score ? d : a));
  const dayName = new Intl.DateTimeFormat(lang, { weekday: "long" });
  const fmt = (n: number) => n.toLocaleString(lang, { maximumFractionDigits: 1 });

  const stats = [
    { icon: "🎯", value: `${fmt(focusH)} ${t.unitHour}`, label: t.focus },
    { icon: "🚶", value: `${breaks}`, label: t.breaksLabel },
    { icon: "💧", value: `${fmt(waterL)} L`, label: t.water },
    { icon: "✅", value: `${goodDays}/7`, label: t.goodDays },
  ];

  const copy = async () => {
    const lines = [
      `${t.weekTitle} — Mola`,
      ...stats.map((s) => `${s.icon} ${s.label}: ${s.value}`),
      `🔥 ${t.streak(streak)}`,
      best.score > 0 ? `⭐ ${t.bestDay}: ${dayName.format(new Date(`${best.date}T00:00`))} (${best.score})` : "",
    ].filter(Boolean);
    const textValue = lines.join("\n");
    try {
      await navigator.clipboard.writeText(textValue);
    } catch {
      const area = document.createElement("textarea");
      area.value = textValue;
      document.body.appendChild(area);
      area.select();
      document.execCommand("copy");
      area.remove();
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="week-summary">
      <div className="panel-head">
        <h2>{t.weekTitle}</h2>
        <span>🔥 {t.streak(streak)}</span>
      </div>
      <div className="summary-grid">
        {stats.map((s) => (
          <div key={s.label} className="summary-stat">
            <span className="summary-value">
              {s.icon} {s.value}
            </span>
            <span className="summary-label">{s.label}</span>
          </div>
        ))}
      </div>
      {best.score > 0 && (
        <p>
          ⭐ {t.bestDay}: <strong>{dayName.format(new Date(`${best.date}T00:00`))}</strong> ({best.score})
        </p>
      )}
      <button className="share" onClick={copy}>
        {copied ? t.copied : t.copySummary}
      </button>
    </section>
  );
}
