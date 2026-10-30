import { useEffect } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { CalEvent, Calendar } from "../lib/api";

interface Props {
  event: CalEvent;
  calendar?: Calendar;
  onClose: () => void;
}

/** Google descriptions are HTML; show them as plain text rather than injecting markup. */
function htmlToText(html: string): string {
  const withBreaks = html.replace(/<br\s*\/?>/gi, "\n").replace(/<\/p>/gi, "\n");
  return new DOMParser().parseFromString(withBreaks, "text/html").body.textContent?.trim() ?? "";
}

function formatRange(e: CalEvent): string {
  const start = new Date(e.start);
  if (e.allDay) {
    const opts: Intl.DateTimeFormatOptions = { weekday: "short", day: "numeric", month: "short", year: "numeric" };
    // Google's all-day end date is exclusive.
    const end = e.end ? new Date(new Date(e.end).getTime() - 86_400_000) : start;
    const s = start.toLocaleDateString(undefined, opts);
    return end.toDateString() === start.toDateString() ? s : `${s} – ${end.toLocaleDateString(undefined, opts)}`;
  }
  const date = start.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short", year: "numeric" });
  const time = (d: Date) => d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", hour12: false });
  return e.end ? `${date}, ${time(start)} – ${time(new Date(e.end))}` : `${date}, ${time(start)}`;
}

export default function EventDetails({ event, calendar, onClose }: Props) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const description = event.description ? htmlToText(event.description) : "";

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <span className="dot" style={{ background: calendar?.backgroundColor }} />
          <h3>{event.title}</h3>
          <button className="icon" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>
        <p className="muted">{formatRange(event)}</p>
        {calendar && <p className="muted small">{calendar.summary}</p>}
        {event.location && <p>📍 {event.location}</p>}
        {description && <p className="description">{description}</p>}
        <div className="modal-actions">
          {event.meetLink && (
            <button className="primary" onClick={() => openUrl(event.meetLink!)}>
              Join Google Meet
            </button>
          )}
          {event.htmlLink && (
            <button onClick={() => openUrl(event.htmlLink!)}>Open in Google Calendar</button>
          )}
        </div>
      </div>
    </div>
  );
}
