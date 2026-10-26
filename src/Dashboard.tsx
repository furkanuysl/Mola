import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, HistoryDay, Settings, Status } from "./api";
import { useLang, useT } from "./i18n";

export function Dashboard() {
  const t = useT();
  const [status, setStatus] = useState<Status | null>(null);
  const [history, setHistory] = useState<HistoryDay[]>([]);

  useEffect(() => {
    api.getStatus().then(setStatus);
    const unlisten = listen<Status>("status", (e) => setStatus(e.payload));
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const glasses = status?.glasses;
  const breaks = status?.breaks;
  useEffect(() => {
    api.getHistory(7).then(setHistory);
  }, [glasses, breaks]);

  if (!status) return null;

  const waterPct = Math.min(100, (status.waterMl / status.waterGoal) * 100);

  return (
    <main className="dashboard">
      <header className="top">
        <h1>
          <img src="/icon.svg" alt="" className="logo" />
          {t.appName}
        </h1>
        {status.pausedMin !== null ? (
          <button className="secondary" onClick={api.resume}>
            {t.resume} <small>{t.minutesLeft(status.pausedMin)}</small>
          </button>
        ) : (
          <button className="secondary" onClick={() => api.pause(60)}>
            {t.pauseHour}
          </button>
        )}
      </header>

      {!status.inWorkHours && <div className="notice">{t.outsideHours}</div>}
      {status.busy && (
        <div className="notice">{status.busy === "call" ? `🎧 ${t.busyCall}` : `🖥️ ${t.busyFullscreen}`}</div>
      )}

      <section className="cards">
        <article className="card water">
          <h2>💧 {t.water}</h2>
          <p className="big">
            {status.waterMl} <small>/ {status.waterGoal} ml</small>
          </p>
          <div className="bar">
            <span style={{ width: `${waterPct}%` }} />
          </div>
          <p className="muted">{status.nextWaterMin !== null ? t.nextWater(status.nextWaterMin) : t.off}</p>
          <button className="primary" onClick={api.addWater}>
            {t.iDrank}
          </button>
        </article>

        <article className="card move">
          <h2>🚶 {t.move}</h2>
          <p className="big">
            {status.sittingMin} <small>{t.sittingFor}</small>
          </p>
          <p className="muted">{t.todaySummary(status.breaks, status.snoozes)}</p>
          <p className="muted">{status.nextMoveMin !== null ? t.nextBreak(status.nextMoveMin) : t.off}</p>
          <button className="primary" onClick={api.takeBreak}>
            {t.takeBreakNow}
          </button>
        </article>
      </section>

      <Week history={history} goal={status.waterGoal} />
      <SettingsForm />
    </main>
  );
}

function Week({ history, goal }: { history: HistoryDay[]; goal: number }) {
  const t = useT();
  const dayName = new Intl.DateTimeFormat(useLang(), { weekday: "short" });
  return (
    <section className="panel">
      <h2>{t.last7Days}</h2>
      <div className="week">
        {history.map((d) => {
          const pct = Math.min(100, (d.waterMl / goal) * 100);
          return (
            <div key={d.date} className="day" title={t.dayTooltip(d.date, d.waterMl, d.breaks)}>
              <div className="col">
                <span className={pct >= 100 ? "met" : ""} style={{ height: `${pct}%` }} />
              </div>
              <span className="breaks">{d.breaks}🚶</span>
              <span className="name">{dayName.format(new Date(`${d.date}T00:00`))}</span>
            </div>
          );
        })}
      </div>
    </section>
  );
}

type NumKey = { [K in keyof Settings]: Settings[K] extends number ? K : never }[keyof Settings];
type BoolKey = { [K in keyof Settings]: Settings[K] extends boolean ? K : never }[keyof Settings];

function SettingsForm() {
  const t = useT();
  const [s, setS] = useState<Settings | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api.getSettings().then(setS);
  }, []);

  if (!s) return null;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setS({ ...s, [key]: value });
    setSaved(false);
  };
  const num = (key: NumKey, label: string, unit: string, min = 1) => (
    <label className="field">
      <span>{label}</span>
      <span className="unit">
        <input type="number" min={min} value={s[key]} onChange={(e) => set(key, Number(e.target.value))} />
        <em>{unit}</em>
      </span>
    </label>
  );
  const toggle = (key: BoolKey, label: string) => (
    <label className="toggle">
      <input type="checkbox" checked={s[key]} onChange={(e) => set(key, e.target.checked)} />
      <span>{label}</span>
    </label>
  );

  const save = async () => {
    setS(await api.saveSettings(s));
    setSaved(true);
  };

  return (
    <section className="panel">
      <h2>{t.settings}</h2>

      <fieldset>
        <legend>{t.water}</legend>
        {toggle("waterEnabled", t.waterEnabled)}
        {num("waterIntervalMin", t.waterInterval, t.unitMin)}
        {num("waterGoalMl", t.waterGoal, t.unitMl)}
        {num("glassMl", t.glass, t.unitMl)}
      </fieldset>

      <fieldset>
        <legend>{t.move}</legend>
        {toggle("moveEnabled", t.moveEnabled)}
        {num("moveIntervalMin", t.moveInterval, t.unitMin)}
        {num("breakMin", t.breakLength, t.unitMin)}
        {num("idleResetMin", t.idleReset, t.unitMin)}
      </fieldset>

      <fieldset>
        <legend>{t.snoozeSection}</legend>
        {num("snoozeMin", t.snoozeLength, t.unitMin)}
        {num("maxSnoozes", t.maxSnoozes, t.unitTimes, 0)}
      </fieldset>

      <fieldset>
        <legend>{t.workHours}</legend>
        {toggle("workHoursEnabled", t.workHoursEnabled)}
        {s.workHoursEnabled && (
          <>
            <label className="field">
              <span>{t.startEnd}</span>
              <span className="unit">
                <input type="time" value={s.workStart} onChange={(e) => set("workStart", e.target.value)} />
                <input type="time" value={s.workEnd} onChange={(e) => set("workEnd", e.target.value)} />
              </span>
            </label>
            {toggle("weekdaysOnly", t.weekdaysOnly)}
          </>
        )}
      </fieldset>

      <fieldset>
        <legend>{t.doNotDisturb}</legend>
        {toggle("quietInFullscreen", t.quietInFullscreen)}
        {toggle("quietInCalls", t.quietInCalls)}
      </fieldset>

      <fieldset>
        <legend>{t.general}</legend>
        {toggle("autostart", t.autostart)}
        <label className="field">
          <span>{t.language}</span>
          <select value={s.language} onChange={(e) => set("language", e.target.value as Settings["language"])}>
            <option value="auto">{t.languageAuto}</option>
            <option value="tr">Türkçe</option>
            <option value="en">English</option>
          </select>
        </label>
      </fieldset>

      <div className="save-row">
        <button className="primary" onClick={save}>
          {t.save}
        </button>
        {saved && <span className="muted">{t.saved}</span>}
      </div>
    </section>
  );
}
