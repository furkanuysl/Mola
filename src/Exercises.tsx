import { useEffect, useState } from "react";
import { BREATHS, Breath, EXERCISES, Routine, ROUTINES, routineSecs, text } from "./exerciseData";
import { useLang, useT } from "./i18n";

function formatClock(secs: number) {
  return `${Math.floor(secs / 60)}:${(secs % 60).toString().padStart(2, "0")}`;
}

type Active = { routine: Routine } | { breath: Breath };

/** Hatırlatmadan bağımsız, istenildiğinde açılan egzersiz rutinleri ve nefes egzersizleri. */
export function Exercises({ onBack }: { onBack: () => void }) {
  const t = useT();
  const lang = useLang();
  const [active, setActive] = useState<Active | null>(null);
  const title = !active ? t.exercises : "routine" in active ? text(active.routine, lang).name : text(active.breath, lang).name;

  return (
    <main className="dashboard">
      <header className="top sub">
        <button className="ghost back" onClick={active ? () => setActive(null) : onBack}>
          ← {t.back}
        </button>
        <h1>{title}</h1>
      </header>

      {active && "routine" in active && <Player routine={active.routine} onExit={() => setActive(null)} />}
      {active && "breath" in active && <BreathPlayer breath={active.breath} onExit={() => setActive(null)} />}
      {!active && (
        <>
          <p className="muted">{t.exercisesIntro}</p>
          <h2 className="section-title">🌬️ {t.breathing}</h2>
          <section className="routines">
            {BREATHS.map((b) => (
              <button key={b.id} className="routine breath" onClick={() => setActive({ breath: b })}>
                <span className="routine-icon">{b.icon}</span>
                <span className="routine-text">
                  <strong>{text(b, lang).name}</strong>
                  <span className="muted">{text(b, lang).desc}</span>
                  <small className="muted">{t.breathMeta(b.pattern.filter((n) => n > 0).join("-"), b.cycles)}</small>
                </span>
              </button>
            ))}
          </section>
          <h2 className="section-title">🧘 {t.routines}</h2>
          <section className="routines">
            {ROUTINES.map((r) => (
              <button key={r.id} className="routine" onClick={() => setActive({ routine: r })}>
                <span className="routine-icon">{r.icon}</span>
                <span className="routine-text">
                  <strong>{text(r, lang).name}</strong>
                  <span className="muted">{text(r, lang).desc}</span>
                  <small className="muted">
                    {t.routineMeta(r.exercises.length, Math.ceil(routineSecs(r) / 60))}
                  </small>
                </span>
              </button>
            ))}
          </section>
          <p className="note safety">⚠️ {t.safetyNote}</p>
        </>
      )}
    </main>
  );
}

/** Nefes döngüsü: daire nefes alırken büyür, verirken küçülür; tutuşlarda sabit kalır. */
function BreathPlayer({ breath, onExit }: { breath: Breath; onExit: () => void }) {
  const t = useT();
  const steps = breath.pattern
    .map((secs, i) => ({ secs, step: (["inhale", "hold", "exhale", "holdOut"] as const)[i] }))
    .filter((s) => s.secs > 0);
  const [cycle, setCycle] = useState(0);
  const [index, setIndex] = useState(0);
  const [left, setLeft] = useState(steps[0].secs);
  const done = cycle >= breath.cycles;

  useEffect(() => {
    if (done) return;
    const id = setInterval(() => setLeft((l) => l - 1), 1000);
    return () => clearInterval(id);
  }, [done]);

  useEffect(() => {
    if (left > 0 || done) return;
    const next = index + 1;
    if (next < steps.length) {
      setIndex(next);
      setLeft(steps[next].secs);
    } else {
      setCycle((c) => c + 1);
      setIndex(0);
      setLeft(steps[0].secs);
    }
  }, [left]);

  if (done) {
    return (
      <section className="panel player done">
        <p className="player-icon">🌿</p>
        <h2>{t.breathDone}</h2>
        <div className="actions">
          <button className="primary" onClick={() => setCycle(0)}>
            {t.again}
          </button>
          <button className="secondary" onClick={onExit}>
            {t.close}
          </button>
        </div>
      </section>
    );
  }

  const current = steps[index];
  // Nefes alırken daire büyür, verirken küçülür; tutuşta önceki boyutunda kalır.
  const big = current.step === "inhale" || current.step === "hold";
  return (
    <section className="panel player breath-player">
      <p className="muted">{t.cycle(cycle + 1, breath.cycles)}</p>
      <div className="breath-stage">
        <div
          className={`breath-circle ${big ? "big" : ""}`}
          style={{ transitionDuration: `${current.step === "inhale" || current.step === "exhale" ? current.secs : 0.3}s` }}
        />
        <div className="breath-label">
          <strong>{t.breathSteps[current.step]}</strong>
          <span>{left}</span>
        </div>
      </div>
      <div className="actions">
        <button className="secondary" onClick={onExit}>
          {t.stop}
        </button>
      </div>
    </section>
  );
}

function Player({ routine, onExit }: { routine: Routine; onExit: () => void }) {
  const t = useT();
  const lang = useLang();
  const [index, setIndex] = useState(0);
  const [left, setLeft] = useState(EXERCISES[routine.exercises[0]].secs);
  const [running, setRunning] = useState(true);
  const done = index >= routine.exercises.length;

  const goTo = (i: number) => {
    setIndex(i);
    if (i < routine.exercises.length) setLeft(EXERCISES[routine.exercises[i]].secs);
  };

  useEffect(() => {
    if (!running || done) return;
    const id = setInterval(() => setLeft((l) => l - 1), 1000);
    return () => clearInterval(id);
  }, [running, done]);

  useEffect(() => {
    if (left <= 0 && !done) goTo(index + 1);
  }, [left]);

  if (done) {
    return (
      <section className="panel player done">
        <p className="player-icon">✅</p>
        <h2>{t.routineDone}</h2>
        <p className="muted">{t.routineDoneBody}</p>
        <div className="actions">
          <button className="primary" onClick={() => goTo(0)}>
            {t.again}
          </button>
          <button className="secondary" onClick={onExit}>
            {t.close}
          </button>
        </div>
      </section>
    );
  }

  const ex = EXERCISES[routine.exercises[index]];
  const exText = text(ex, lang);
  return (
    <section className="panel player">
      <p className="muted">
        {t.step(index + 1, routine.exercises.length)}
      </p>
      <h2>
        {ex.icon} {exText.name}
      </h2>
      <div className="clock">{formatClock(left)}</div>
      <div className="bar">
        <span style={{ width: `${((ex.secs - left) / ex.secs) * 100}%` }} />
      </div>
      <ol className="steps">
        {exText.steps.map((s) => (
          <li key={s}>{s}</li>
        ))}
      </ol>
      <div className="actions">
        <button className="secondary" disabled={index === 0} onClick={() => goTo(index - 1)}>
          ‹ {t.previous}
        </button>
        <button className="primary" onClick={() => setRunning(!running)}>
          {running ? t.pauseExercise : t.continueExercise}
        </button>
        <button className="secondary" onClick={() => goTo(index + 1)}>
          {t.next} ›
        </button>
      </div>
    </section>
  );
}
