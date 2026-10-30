import { useCallback, useEffect, useMemo, useState } from "react";
import { api, AUTH_EXPIRED, Calendar } from "./lib/api";
import Sidebar from "./components/Sidebar";
import CalendarView from "./components/CalendarView";
import "./App.css";

type Status = "loading" | "signedOut" | "signingIn" | "signedIn";

const HIDDEN_KEY = "hiddenCalendars";

function loadHidden(): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem(HIDDEN_KEY) ?? "[]"));
  } catch {
    return new Set();
  }
}

export default function App() {
  const [status, setStatus] = useState<Status>("loading");
  const [error, setError] = useState<string | null>(null);
  const [calendars, setCalendars] = useState<Calendar[]>([]);
  const [hidden, setHidden] = useState<Set<string>>(loadHidden);
  const [background, setBackground] = useState<string | null>(null);

  const handleError = useCallback((e: unknown) => {
    if (e === AUTH_EXPIRED) {
      setCalendars([]);
      setStatus("signedOut");
      setError("Your session expired. Please sign in again.");
    } else {
      setError(String(e));
    }
  }, []);

  const loadCalendars = useCallback(async () => {
    try {
      setCalendars(await api.listCalendars());
      setStatus("signedIn");
      setError(null);
    } catch (e) {
      handleError(e);
    }
  }, [handleError]);

  useEffect(() => {
    api.getBackground().then(setBackground).catch(() => {});
  }, []);

  const chooseBackground = async () => {
    try {
      const url = await api.pickBackground();
      if (url) setBackground(url);
    } catch (e) {
      setError(String(e));
    }
  };

  const removeBackground = async () => {
    await api.clearBackground().catch((e) => setError(String(e)));
    setBackground(null);
  };

  const bgStyle = background ? { backgroundImage: `url("${background}")` } : undefined;
  const bgClass = background ? " has-bg" : "";

  useEffect(() => {
    api.isLoggedIn().then((ok) => (ok ? loadCalendars() : setStatus("signedOut")));
  }, [loadCalendars]);

  const signIn = async () => {
    setStatus("signingIn");
    setError(null);
    try {
      await api.login();
      await loadCalendars();
    } catch (e) {
      setStatus("signedOut");
      setError(String(e));
    }
  };

  const signOut = async () => {
    await api.logout().catch(() => {});
    setCalendars([]);
    setStatus("signedOut");
  };

  const toggleCalendar = (id: string) => {
    setHidden((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      try {
        localStorage.setItem(HIDDEN_KEY, JSON.stringify([...next]));
      } catch {
        /* storage unavailable — visibility just won't persist */
      }
      return next;
    });
  };

  const visibleCalendars = useMemo(
    () => calendars.filter((c) => !hidden.has(c.id)),
    [calendars, hidden],
  );

  if (status === "loading") {
    return (
      <div className={`center muted${bgClass}`} style={bgStyle}>
        Loading…
      </div>
    );
  }

  if (status !== "signedIn") {
    return (
      <div className={`center${bgClass}`} style={bgStyle}>
        <div className="login-card">
          <h1>Calendar</h1>
          <p className="muted">A read-only view of your Google Calendar.</p>
          <button className="primary" onClick={signIn} disabled={status === "signingIn"}>
            {status === "signingIn" ? "Waiting for browser…" : "Sign in with Google"}
          </button>
          {status === "signingIn" && (
            <p className="muted small">Finish signing in in the browser window that opened.</p>
          )}
          {error && <p className="error">{error}</p>}
        </div>
      </div>
    );
  }

  return (
    <div className={`layout${bgClass}`} style={bgStyle}>
      <Sidebar
        calendars={calendars}
        hidden={hidden}
        onToggle={toggleCalendar}
        onSignOut={signOut}
        hasBackground={background !== null}
        onChooseBackground={chooseBackground}
        onRemoveBackground={removeBackground}
      />
      <main className="main">
        {error && (
          <div className="banner error" onClick={() => setError(null)}>
            {error}
          </div>
        )}
        <CalendarView calendars={visibleCalendars} onError={handleError} />
      </main>
    </div>
  );
}
