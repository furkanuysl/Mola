import { invoke } from "@tauri-apps/api/core";
import type { Lang } from "./i18n";

export type Kind = "water" | "move";

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
  language: "auto" | Lang;
}

export interface HistoryDay {
  date: string;
  waterMl: number;
  glasses: number;
  breaks: number;
  snoozes: number;
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
};
