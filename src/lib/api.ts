import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface Calendar {
  id: string;
  summary: string;
  backgroundColor: string;
  foregroundColor: string;
  primary: boolean;
}

export interface CalEvent {
  id: string;
  calendarId: string;
  title: string;
  start: string;
  end: string | null;
  allDay: boolean;
  location: string | null;
  description: string | null;
  htmlLink: string | null;
  meetLink: string | null;
}

/** Error string the backend returns when the user must sign in again. */
export const AUTH_EXPIRED = "AUTH_EXPIRED";

export const api = {
  isLoggedIn: () => invoke<boolean>("is_logged_in"),
  login: () => invoke<void>("login"),
  logout: () => invoke<void>("logout"),
  listCalendars: () => invoke<Calendar[]>("list_calendars"),
  listEvents: (calendarIds: string[], timeMin: string, timeMax: string) =>
    invoke<CalEvent[]>("list_events", { calendarIds, timeMin, timeMax }),
  getBackground: () => invoke<string | null>("get_background"),
  clearBackground: () => invoke<void>("clear_background"),
  /** Opens a file picker; returns the new background as a data URL, or null if cancelled. */
  pickBackground: async () => {
    const path = await open({
      title: "Choose a background picture",
      multiple: false,
      directory: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }],
    });
    return path ? invoke<string>("set_background", { path }) : null;
  },
};
