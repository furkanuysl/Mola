import { invoke } from "@tauri-apps/api/core";
import type { Lang } from "./i18n";

export type Kind = "water" | "move" | "eyes";
export type Technique = "pomodoro" | "52_17" | "90_20" | "flowtime";

export interface FocusView {
  technique: Technique;
  phase: "off" | "focus" | "break" | "ready";
  elapsed: number;
  target: number | null;
  paused: boolean;
  round: number;
  rounds: number | null;
  breakRemaining: number | null;
}

export interface ReminderView {
  kind: Kind;
  phase: "prompt" | "break" | "done";
  breakRemaining: number;
  breakTotal: number;
  snoozesLeft: number;
  snoozeMin: number;
  glassMl: number;
  waterToday: number;
  waterGoal: number;
  sittingMin: number;
  breakRoutine: string;
  focusBreak: boolean;
  focusNext: "" | "ready" | "started";
}

export interface Status {
  waterMl: number;
  waterGoal: number;
  glasses: number;
  breaks: number;
  snoozes: number;
  sittingMin: number;
  nextWaterMin: number | null;
  nextMoveMin: number | null;
  pausedMin: number | null;
  inWorkHours: boolean;
  busy: "fullscreen" | "call" | null;
  focusMin: number;
  score: number;
  streak: number;
  focus: FocusView;
}

export interface Settings {
  waterEnabled: boolean;
  waterIntervalMin: number;
  waterGoalMl: number;
  glassMl: number;
  moveEnabled: boolean;
  moveIntervalMin: number;
  breakMin: number;
  idleResetMin: number;
  snoozeMin: number;
  maxSnoozes: number;
  workHoursEnabled: boolean;
  workStart: string;
  workEnd: string;
  weekdaysOnly: boolean;
  autostart: boolean;
  quietInFullscreen: boolean;
  quietInCalls: boolean;
  language: "auto" | Lang;
  breakRoutine: string;
  eyesEnabled: boolean;
  eyesIntervalMin: number;
  focusTechnique: Technique;
  pomodoroWorkMin: number;
  pomodoroShortMin: number;
  pomodoroLongMin: number;
  pomodoroRounds: number;
  focusAutoContinue: boolean;
  breaksGoal: number;
  focusGoalMin: number;
}

export interface HistoryDay {
  date: string;
  waterMl: number;
  glasses: number;
  breaks: number;
  snoozes: number;
  focusSecs: number;
  focusSessions: number;
  score: number;
}

export type ReminderAction = "drink" | "skip" | "snooze" | "start_break" | "dismiss";

export const api = {
  getReminder: () => invoke<ReminderView | null>("get_reminder"),
  reminderAction: (action: ReminderAction) => invoke<void>("reminder_action", { action }),
  getStatus: () => invoke<Status>("get_status"),
  getSettings: () => invoke<Settings>("get_settings"),
  getLanguage: () => invoke<Lang>("get_language"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  getHistory: (days: number) => invoke<HistoryDay[]>("get_history", { days }),
  addWater: () => invoke<void>("add_water"),
  takeBreak: () => invoke<void>("take_break"),
  pause: (minutes: number) => invoke<void>("pause", { minutes }),
  resume: () => invoke<void>("resume"),
  focusStart: (technique: Technique) => invoke<void>("focus_start", { technique }),
  focusPause: (paused: boolean) => invoke<void>("focus_pause", { paused }),
  focusStop: () => invoke<void>("focus_stop"),
  focusBreakNow: () => invoke<void>("focus_break_now"),
};
