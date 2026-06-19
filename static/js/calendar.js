/**
 * Calendar view for daily events, powered by FullCalendar.
 *
 * Renders the same filtered daily summaries as the spreadsheet (see
 * EventSpreadsheet.filteredData) as month-grid event chips, colored to match
 * the table's meal-type / activity-type palette. Event click opens the item
 * for editing (reusing EventSpreadsheet); clicking an empty day opens the
 * "add meal" form for that date.
 */
class CalendarView {
    constructor(containerId) {
        this.containerId = containerId;
        this.cal = null;
        this.monthData = [];        // raw (merged) summaries for the visible range
        this.data = [];             // monthData after keyword/activity-type filters
        this.initialized = false;
        this._loadSeq = 0;          // guards against out-of-order range loads
    }

    /**
     * Lazily create the FullCalendar instance. Must be called only while the
     * container is visible so sizing is correct.
     *
     * Unlike the table, the calendar ignores the date-range filter: it loads
     * data for whatever range is currently visible (see onDatesSet) so the user
     * can scroll month-to-month seamlessly. It opens on the table's end date for
     * continuity, falling back to today.
     */
    init() {
        if (this.initialized) return;

        const el = document.getElementById(this.containerId);
        if (!el || typeof FullCalendar === 'undefined') return;

        const f = (window.eventSpreadsheet && window.eventSpreadsheet.currentFilters) || {};
        const initialDate = f.endDate || (window.dateUtils && window.dateUtils.getTodayLocal());

        this.cal = new FullCalendar.Calendar(el, {
            initialView: 'dayGridMonth',
            initialDate: initialDate,
            height: window.innerHeight - 210,
            headerToolbar: { left: 'prev,next today', center: 'title', right: '' },
            dayMaxEvents: true,           // collapse crowded days into "+N more"
            displayEventEnd: false,
            eventDisplay: 'block',
            eventTimeFormat: false,       // all-day chips; no time text
            titleFormat: { year: 'numeric', month: 'long' },
            dateClick: (info) => this.onDateClick(info),
            eventClick: (info) => this.onEventClick(info),
            // Fires on first render and on every prev/next/today navigation.
            datesSet: (info) => this.onDatesSet(info),
        });

        this.cal.render();
        this.initialized = true;
    }

    /**
     * Re-apply the keyword / activity-type filters to the current month's data
     * and refresh the rendered chips. The date-range filter is intentionally
     * ignored here. No-op until the calendar is mounted.
     */
    update() {
        if (!this.cal) return;
        this.render();
    }

    /**
     * Refetch the currently visible range from the API. Used when the underlying
     * data may have changed (edits/saves/refresh). No-op until mounted.
     */
    refresh() {
        if (!this.cal) return;
        const view = this.cal.view;
        const start = this.cal.formatIso(view.activeStart, true);
        const end = this.cal.formatIso(view.activeEnd, true);
        this.loadRange(start, end);
    }

    /**
     * Load the daily summaries for the visible date range, then render. Called by
     * FullCalendar whenever the visible month changes, so scrolling fetches fresh
     * data instead of leaving empty pages outside the table's date range.
     */
    onDatesSet(info) {
        // info.startStr/endStr cover the full visible grid (incl. adjacent-month
        // days). endStr is exclusive; the API tolerates the extra trailing day.
        const start = (info.startStr || '').slice(0, 10);
        const end = (info.endStr || '').slice(0, 10);
        if (start && end) this.loadRange(start, end);
    }

    /**
     * Fetch summaries for [start, end], merge meals like the table does, and
     * render. A sequence guard discards responses from superseded loads.
     */
    async loadRange(start, end) {
        const seq = ++this._loadSeq;
        try {
            const summaries = await apiClient.getDailySummary(start, end);
            if (seq !== this._loadSeq) return;   // a newer load started; discard

            const es = window.eventSpreadsheet;
            this.monthData = (es && es.postProcessMealMerging)
                ? es.postProcessMealMerging(summaries || [])
                : (summaries || []);
            this.render();
        } catch (err) {
            console.error('Calendar failed to load range', start, end, err);
        }
    }

    /**
     * Filter the loaded month data by the active keyword / activity-type filters
     * (reusing the table's predicates) and rebuild the calendar event source.
     */
    render() {
        if (!this.cal) return;
        this.data = this.filterDays(this.monthData || []);
        this.cal.getEventSources().forEach((s) => s.remove());
        this.cal.addEventSource(this.buildEvents(this.data));
    }

    /**
     * Keep only days that have a hit for the active keyword + activity-type
     * filters, mirroring EventSpreadsheet.applyFilters' row-level filtering.
     */
    filterDays(days) {
        const es = window.eventSpreadsheet;
        const term = this.getSearchTerm();
        const activityType = this.getActivityType();
        return days.filter((day) => {
            if (!day || !day.date) return false;
            if (term && es && !es.rowMatchesSearch(day, term)) return false;
            if (activityType && es && !es.rowMatchesActivityType(day, activityType)) return false;
            return true;
        });
    }

    /** Current keyword-search term (lowercased, trimmed), or '' when none. */
    getSearchTerm() {
        const f = (window.eventSpreadsheet && window.eventSpreadsheet.currentFilters) || {};
        return (f.searchText || '').toLowerCase().trim();
    }

    /** Current activity-type filter (lowercased, trimmed), or '' when none. */
    getActivityType() {
        const f = (window.eventSpreadsheet && window.eventSpreadsheet.currentFilters) || {};
        return (f.activityType || '').toLowerCase().trim();
    }

    textMatches(text, term) {
        return !term || (!!text && text.toLowerCase().includes(term));
    }

    /**
     * Convert a filtered daily-summary array into FullCalendar event objects.
     */
    buildEvents(days) {
        const term = this.getSearchTerm();
        const events = [];

        for (const day of days) {
            if (!day || !day.date) continue;
            events.push(...this.buildDayEvents(day, term));
        }

        return events;
    }

    /**
     * Build events for a single day. When a keyword search is active, only the
     * matching items are rendered. (Days without any hit are already excluded
     * upstream by EventSpreadsheet.applyFilters.) If a day matched only on its
     * date/day text rather than an item, fall back to showing all its items.
     */
    buildDayEvents(day, term) {
        const date = day.date;
        const activityType = this.getActivityType();

        // Gather items with a per-item match flag.
        const meals = [];
        for (const mealTime of ['breakfast', 'lunch', 'dinner']) {
            for (const meal of (day[mealTime] || [])) {
                const text = (window.eventSpreadsheet && window.eventSpreadsheet.getMealDisplayText)
                    ? window.eventSpreadsheet.getMealDisplayText(meal)
                    : (meal.name || '');
                meals.push({ mealTime, meal, match: this.textMatches(text, term) });
            }
        }

        const drinks = (day.drinks || []).map((d) => ({ d, match: this.textMatches(d, term) }));

        const activities = (day.events || []).map((ev) => ({
            ev,
            match: this.textMatches(ev.text, term),
            typeMatch: !activityType || (ev.type || '').toLowerCase() === activityType,
        }));

        const anyItemMatch = !term
            || meals.some((m) => m.match)
            || drinks.some((d) => d.match)
            || activities.some((a) => a.match);
        // Only filter to matched items if the search actually hit an item on this day.
        const filterItems = term !== '' && anyItemMatch;

        const out = [];

        // When an activity-type filter is active, only the matching activity
        // events are shown; meals and drinks have no activity type and are hidden.
        if (!activityType) {
            for (const m of meals) {
                if (!filterItems || m.match) {
                    out.push(this.mealEvent(date, m.mealTime, m.meal));
                }
            }

            const drinkNames = (filterItems ? drinks.filter((d) => d.match) : drinks).map((d) => d.d);
            if (drinkNames.length) {
                out.push({
                    title: drinkNames.join(', '),
                    start: date,
                    allDay: true,
                    backgroundColor: CalendarView.DRINK_COLOR,
                    borderColor: CalendarView.DRINK_COLOR,
                    textColor: CalendarView.TEXT_COLOR,
                    extendedProps: { kind: 'drink' },
                });
            }
        }

        for (const a of activities) {
            if (!a.typeMatch) continue;
            if (!filterItems || a.match) {
                out.push(this.activityEvent(date, a.ev));
            }
        }

        return out;
    }

    mealEvent(date, mealTime, meal) {
        const text = (window.eventSpreadsheet && window.eventSpreadsheet.getMealDisplayText)
            ? window.eventSpreadsheet.getMealDisplayText(meal)
            : (meal.name || '');

        const typeKey = (meal.type || '').toLowerCase();
        const bg = CalendarView.MEAL_TYPE_COLORS[typeKey]
            || CalendarView.MEAL_TIME_COLORS[mealTime]
            || '#e8f5e8';

        return {
            title: text || '(meal)',
            start: date,
            allDay: true,
            backgroundColor: bg,
            borderColor: bg,
            textColor: CalendarView.TEXT_COLOR,
            extendedProps: { kind: 'meal', mealIds: meal.ids || [] },
        };
    }

    activityEvent(date, ev) {
        const typeKey = (ev.type || '').replace(/\s+/g, '-').toLowerCase();
        const bg = CalendarView.ACTIVITY_TYPE_COLORS[typeKey] || '#e3f2fd';
        return {
            title: ev.text || '(event)',
            start: date,
            allDay: true,
            backgroundColor: bg,
            borderColor: bg,
            textColor: CalendarView.TEXT_COLOR,
            extendedProps: { kind: 'event', id: ev.id },
        };
    }

    onEventClick(info) {
        const p = info.event.extendedProps || {};
        const es = window.eventSpreadsheet;
        if (!es) return;

        if (p.kind === 'meal') {
            if (!p.mealIds || p.mealIds.length === 0) return;
            if (p.mealIds.length > 1) {
                alert('Editing multiple meals in one entry is not supported. Please use the table view to edit individual meals.');
                return;
            }
            es.editMealById(p.mealIds[0]);
        } else if (p.kind === 'event') {
            if (p.id) es.editEventById(p.id);
        } else if (p.kind === 'drink') {
            alert('Drink editing is not available from the calendar view. Please use the table view or the Drinks page.');
        }
    }

    onDateClick(info) {
        // Mirror the table's empty-cell double-click: open "add meal" for the day.
        if (window.detailForms) {
            window.detailForms.openDetails(null, 'meal', info.dateStr);
        }
    }
}

// Palettes mirror the table (see .meal-type-* / .activity-type-* in main.css).
CalendarView.TEXT_COLOR = '#333333';
CalendarView.DRINK_COLOR = '#fff3e0';
// Fallback colors keyed by meal time.
CalendarView.MEAL_TIME_COLORS = {
    breakfast: '#fff8e1',
    lunch: '#e3f2fd',
    dinner: '#fce4ec',
};
CalendarView.MEAL_TYPE_COLORS = {
    'dine-in': '#fff3e0',
    'cooked': '#e3f2fd',
    'takeout': '#e8f5e8',
    'manufactured': '#f3e5f5',
    'leftover': '#f5f5f5',
};
CalendarView.ACTIVITY_TYPE_COLORS = {
    'work': '#ffebee',
    'chore': '#f3e5f5',
    'entertainment': '#e8f5e8',
    'housekeeping': '#f5f5f5',
    'side-project': '#fff9c4',
    'sport': '#e3f2fd',
    'study': '#fff3e0',
    'transport': '#e0f2f1',
    'vedio-game': '#c8e6c9',
};
