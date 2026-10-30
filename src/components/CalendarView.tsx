import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import FullCalendar, {
  type CalendarRef,
  type EventClickInfo,
  type EventInput,
  type EventSourceFuncInfo,
} from "@fullcalendar/react";
import classicTheme from "@fullcalendar/react/themes/classic";
import dayGridPlugin from "@fullcalendar/react/daygrid";
import timeGridPlugin from "@fullcalendar/react/timegrid";
import listPlugin from "@fullcalendar/react/list";
import "@fullcalendar/react/skeleton.css";
import "@fullcalendar/react/themes/classic/theme.css";
import "@fullcalendar/react/themes/classic/palette.css";
import { api, CalEvent, Calendar } from "../lib/api";
import EventDetails from "./EventDetails";

const REFRESH_MS = 5 * 60 * 1000;

const HEADER_TOOLBAR = {
  left: "prev,next today refresh",
  center: "title",
  right: "dayGridMonth,timeGridWeek,timeGridDay,listWeek",
};
const TIME_FORMAT = { hour: "2-digit", minute: "2-digit", hour12: false } as const;

interface Props {
  calendars: Calendar[];
  onError: (e: unknown) => void;
}

export default function CalendarView({ calendars, onError }: Props) {
  const calRef = useRef<CalendarRef>(null);
  const [selected, setSelected] = useState<CalEvent | null>(null);
  const [loading, setLoading] = useState(false);

  // Every prop handed to FullCalendar must keep its identity across renders: a new `events`
  // function counts as a new event source and triggers a refetch, which toggles `loading`,
  // re-renders, and loops. So the callbacks below are stable and read live values via refs.
  const calendarsRef = useRef(calendars);
  calendarsRef.current = calendars;
  const onErrorRef = useRef(onError);
  onErrorRef.current = onError;

  const refetch = useCallback(() => calRef.current?.getApi().refetchEvents(), []);

  const visibleKey = calendars.map((c) => c.id).join(",");
  useEffect(() => {
    refetch();
  }, [visibleKey, refetch]);

  useEffect(() => {
    const timer = setInterval(refetch, REFRESH_MS);
    window.addEventListener("focus", refetch);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", refetch);
    };
  }, [refetch]);

  const fetchEvents = useCallback(
    (
      info: EventSourceFuncInfo,
      success: (events: EventInput[]) => void,
      failure: (error: Error) => void,
    ) => {
      const cals = calendarsRef.current;
      if (cals.length === 0) {
        success([]);
        return;
      }
      const byId = new Map(cals.map((c) => [c.id, c]));
      api
        .listEvents(
          cals.map((c) => c.id),
          info.start.toISOString(),
          info.end.toISOString(),
        )
        .then((events) =>
          success(
            events.map((e) => {
              const cal = byId.get(e.calendarId);
              return {
                id: `${e.calendarId}:${e.id}`,
                title: e.title,
                start: e.start,
                end: e.end ?? undefined,
                allDay: e.allDay,
                backgroundColor: cal?.backgroundColor,
                borderColor: cal?.backgroundColor,
                textColor: cal?.foregroundColor,
                extendedProps: { raw: e },
              };
            }),
          ),
        )
        .catch((err) => {
          onErrorRef.current(err);
          failure(new Error(String(err)));
        });
    },
    [],
  );

  const handleClick = useCallback((arg: EventClickInfo) => {
    arg.jsEvent.preventDefault();
    setSelected(arg.event.extendedProps.raw as CalEvent);
  }, []);

  const buttons = useMemo(
    () => ({
      today: { text: "Today" },
      month: { text: "Month" },
      week: { text: "Week" },
      day: { text: "Day" },
      listWeek: { text: "Agenda" },
      refresh: { text: "↻", hint: "Refresh", click: refetch },
    }),
    [refetch],
  );

  return (
    <div className={`calendar-wrap${loading ? " is-loading" : ""}`}>
      <FullCalendar
        ref={calRef}
        plugins={[classicTheme, dayGridPlugin, timeGridPlugin, listPlugin]}
        initialView="dayGridMonth"
        headerToolbar={HEADER_TOOLBAR}
        buttons={buttons}
        events={fetchEvents}
        loading={setLoading}
        eventClick={handleClick}
        editable={false}
        nowIndicator
        firstDay={1}
        height="100%"
        dayMaxEvents
        popoverClass="solid-popover"
        eventTimeFormat={TIME_FORMAT}
        slotHeaderFormat={TIME_FORMAT}
      />
      {selected && (
        <EventDetails
          event={selected}
          calendar={calendars.find((c) => c.id === selected.calendarId)}
          onClose={() => setSelected(null)}
        />
      )}
    </div>
  );
}
