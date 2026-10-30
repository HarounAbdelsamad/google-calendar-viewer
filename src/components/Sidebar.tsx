import { Calendar } from "../lib/api";

interface Props {
  calendars: Calendar[];
  hidden: Set<string>;
  onToggle: (id: string) => void;
  onSignOut: () => void;
  hasBackground: boolean;
  onChooseBackground: () => void;
  onRemoveBackground: () => void;
}

export default function Sidebar({
  calendars,
  hidden,
  onToggle,
  onSignOut,
  hasBackground,
  onChooseBackground,
  onRemoveBackground,
}: Props) {
  const sorted = [...calendars].sort(
    (a, b) => Number(b.primary) - Number(a.primary) || a.summary.localeCompare(b.summary),
  );

  return (
    <aside className="sidebar">
      <h2>My calendars</h2>
      <ul className="cal-list">
        {sorted.map((c) => (
          <li key={c.id}>
            <label>
              <input
                type="checkbox"
                checked={!hidden.has(c.id)}
                onChange={() => onToggle(c.id)}
                style={{ accentColor: c.backgroundColor }}
              />
              <span className="cal-name" title={c.summary}>
                {c.summary}
              </span>
            </label>
          </li>
        ))}
      </ul>
      <h2>Background</h2>
      <div className="bg-actions">
        <button onClick={onChooseBackground}>{hasBackground ? "Change picture…" : "Choose picture…"}</button>
        {hasBackground && (
          <button className="link" onClick={onRemoveBackground}>
            Remove
          </button>
        )}
      </div>
      <button className="link" onClick={onSignOut}>
        Sign out
      </button>
    </aside>
  );
}
