/**
 * Main application initialization and event handlers
 */
document.addEventListener('DOMContentLoaded', function () {
    // Initialize components
    window.eventSpreadsheet = new EventSpreadsheet('main-spreadsheet');
    // Calendar view (FullCalendar instance is created lazily on first show)
    window.calendarView = new CalendarView('calendar');

    // Setup control button handlers
    setupControlButtons();

    // Setup filter handlers
    setupFilters();

    // Initialize date filters from URL or defaults
    initializeDateFilters();

    // Populate the activity-type filter dropdown from the API
    populateActivityTypeFilter();

    // Handle browser back/forward buttons
    window.addEventListener('popstate', function (event) {
        // Sync form inputs with URL parameters
        const urlParams = getUrlFilters();
        if (urlParams.startDate) {
            document.getElementById('startDate').value = urlParams.startDate;
        }
        if (urlParams.endDate) {
            document.getElementById('endDate').value = urlParams.endDate;
        }
        if (urlParams.searchText) {
            document.getElementById('searchText').value = urlParams.searchText;
        }
        const activityTypeSelect = document.getElementById('activityTypeFilter');
        if (activityTypeSelect) {
            activityTypeSelect.value = urlParams.activityType || '';
        }

        // Apply the filters from URL
        const filters = {
            startDate: urlParams.startDate || window.dateUtils.getDaysAgoLocal(30),
            endDate: urlParams.endDate || window.dateUtils.getTodayLocal(),
            searchText: urlParams.searchText || '',
            activityType: urlParams.activityType || ''
        };
        window.eventSpreadsheet.setFilters(filters);
    });
});

/**
 * Load the available activity types and populate the filter dropdown,
 * preserving any value already set from the URL.
 */
async function populateActivityTypeFilter() {
    const select = document.getElementById('activityTypeFilter');
    if (!select) return;

    try {
        const activityTypes = await apiClient.getActivityTypes();
        const current = getUrlFilters().activityType || '';

        for (const activityType of (activityTypes || [])) {
            const option = document.createElement('option');
            option.value = activityType.name;
            option.textContent = activityType.name;
            select.appendChild(option);
        }

        // Restore selection from URL after options are populated.
        select.value = current;
    } catch (err) {
        console.warn('Failed to load activity types for filter:', err);
    }
}

function setupControlButtons() {
    // Add event buttons
    document.getElementById('addMealBtn').onclick = () => {
        window.detailForms.openDetails(null, 'meal');
    };

    document.getElementById('addEventBtn').onclick = () => {
        window.detailForms.openDetails(null, 'event');
    };

    document.getElementById('addDrinkBtn').onclick = () => {
        window.detailForms.openDetails(null, 'drink');
    };

    // Refresh button
    document.getElementById('refreshBtn').onclick = () => {
        window.eventSpreadsheet.refreshWithDefaultDates();
    };

    // Toggle columns button
    document.getElementById('toggleColumnsBtn').onclick = () => {
        window.eventSpreadsheet.toggleMealsAndDrinksColumns();
    };

    // View mode toggle (Table / Calendar)
    const viewModeBtn = document.getElementById('viewModeBtn');
    const dailyViewEl = document.querySelector('.daily-view');
    if (viewModeBtn && dailyViewEl) {
        viewModeBtn.onclick = () => {
            const showCalendar = !dailyViewEl.classList.contains('view-calendar');
            dailyViewEl.classList.toggle('view-calendar', showCalendar);
            viewModeBtn.title = showCalendar ? 'Switch to table view' : 'Switch to calendar view';

            // The date range only drives the table; the calendar loads per visible
            // month, so disable the date inputs while the calendar is showing.
            setDateFilterEnabled(!showCalendar);

            if (showCalendar && window.calendarView) {
                // Container must be visible before init so FullCalendar sizes correctly.
                window.calendarView.init();
                window.calendarView.update();
                if (window.calendarView.cal) {
                    window.calendarView.cal.updateSize();
                }
            }
        };
    }

    // Clear search button
    document.getElementById('clearSearchBtn').onclick = () => {
        const searchInput = document.getElementById('searchText');
        const clearButton = document.getElementById('clearSearchBtn');

        searchInput.value = '';

        // Hide the clear button immediately
        clearButton.classList.remove('visible');

        // Trigger the search to clear results immediately
        const filters = {
            searchText: ''
        };
        window.eventSpreadsheet.setFilters(filters);

        // Update URL to remove search parameter
        updateUrlFilters(
            document.getElementById('startDate').value,
            document.getElementById('endDate').value,
            ''
        );
    };

    // Navigation dropdown toggle
    const navDropdownToggle = document.querySelector('.nav-dropdown-toggle');
    const navDropdownMenu = document.querySelector('.nav-dropdown-menu');

    if (navDropdownToggle && navDropdownMenu) {
        navDropdownToggle.onclick = (e) => {
            e.stopPropagation();
            navDropdownMenu.classList.toggle('show');
        };

        // Close dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!navDropdownToggle.contains(e.target) && !navDropdownMenu.contains(e.target)) {
                navDropdownMenu.classList.remove('show');
            }
        });

        // Close dropdown when clicking on menu items
        navDropdownMenu.addEventListener('click', (e) => {
            if (e.target.tagName === 'A') {
                navDropdownMenu.classList.remove('show');
            }
        });
    }
}

/**
 * Enable/disable the date-range inputs. They drive the table only; the calendar
 * view loads its own per-month data and ignores them, so they are disabled and
 * dimmed while the calendar is showing.
 */
function setDateFilterEnabled(enabled) {
    ['startDate', 'endDate'].forEach((id) => {
        const input = document.getElementById(id);
        if (!input) return;
        input.disabled = !enabled;
        const group = input.closest('.filter-group');
        if (group) group.classList.toggle('filter-disabled', !enabled);
    });
}

function setupFilters() {
    document.getElementById('applyFilters').onclick = () => {
        const startDate = document.getElementById('startDate').value;
        const endDate = document.getElementById('endDate').value;
        const searchText = document.getElementById('searchText').value;
        const activityType = document.getElementById('activityTypeFilter').value;

        // Update URL with new filter parameters
        updateUrlFilters(startDate, endDate, searchText, true, activityType);

        const filters = {
            startDate: startDate,
            endDate: endDate,
            searchText: searchText,
            activityType: activityType
        };

        window.eventSpreadsheet.setFilters(filters);
    };

    // Apply the activity-type filter immediately on change (client-side only).
    const activityTypeSelect = document.getElementById('activityTypeFilter');
    if (activityTypeSelect) {
        activityTypeSelect.addEventListener('change', () => {
            const activityType = activityTypeSelect.value;
            updateUrlFilters(
                document.getElementById('startDate').value,
                document.getElementById('endDate').value,
                document.getElementById('searchText').value,
                true,
                activityType
            );
            window.eventSpreadsheet.setFilters({ activityType });
        });
    }

    // Add real-time search filtering with debounce
    const searchInput = document.getElementById('searchText');
    const clearButton = document.getElementById('clearSearchBtn');

    // Function to toggle clear button visibility
    const updateClearButtonVisibility = () => {
        if (searchInput.value.trim().length > 0) {
            clearButton.classList.add('visible');
        } else {
            clearButton.classList.remove('visible');
        }
    };

    const debouncedSearch = window.utils.debounce(() => {
        const searchText = searchInput.value;
        const filters = {
            searchText: searchText
        };
        window.eventSpreadsheet.setFilters(filters);
        updateClearButtonVisibility();
    }, 300);

    searchInput.addEventListener('input', debouncedSearch);

    // Set initial visibility state
    updateClearButtonVisibility();
}


function initializeDateFilters() {
    // Read URL parameters first
    const urlParams = getUrlFilters();

    if (urlParams.startDate && urlParams.endDate && isValidDate(urlParams.startDate) && isValidDate(urlParams.endDate)) {
        // Use URL parameters if they exist and are valid
        document.getElementById('startDate').value = urlParams.startDate;
        document.getElementById('endDate').value = urlParams.endDate;
    } else {
        // Set defaults and update URL
        const defaultStart = window.dateUtils.getDaysAgoLocal(30);
        const defaultEnd = window.dateUtils.getTodayLocal();

        document.getElementById('startDate').value = defaultStart;
        document.getElementById('endDate').value = defaultEnd;

        // Update URL with defaults (without triggering a reload)
        updateUrlFilters(defaultStart, defaultEnd, '', false);
    }

    // Initialize search text from URL
    if (urlParams.searchText) {
        document.getElementById('searchText').value = urlParams.searchText;
    }
}

function isValidDate(dateString) {
    const date = new Date(dateString);
    return date instanceof Date && !isNaN(date) && dateString.match(/^\d{4}-\d{2}-\d{2}$/);
}

function getUrlFilters() {
    const urlParams = new URLSearchParams(window.location.search);
    return {
        startDate: urlParams.get('startDate'),
        endDate: urlParams.get('endDate'),
        searchText: urlParams.get('searchText'),
        activityType: urlParams.get('activityType')
    };
}

// `activityType` is optional: when omitted (undefined) the existing URL value is
// left untouched, so callers that don't manage it won't clobber the selection.
function updateUrlFilters(startDate, endDate, searchText, pushState = true, activityType = undefined) {
    const url = new URL(window.location);

    if (startDate) {
        url.searchParams.set('startDate', startDate);
    } else {
        url.searchParams.delete('startDate');
    }

    if (endDate) {
        url.searchParams.set('endDate', endDate);
    } else {
        url.searchParams.delete('endDate');
    }

    if (searchText && searchText.trim()) {
        url.searchParams.set('searchText', searchText.trim());
    } else {
        url.searchParams.delete('searchText');
    }

    if (activityType !== undefined) {
        if (activityType && activityType.trim()) {
            url.searchParams.set('activityType', activityType.trim());
        } else {
            url.searchParams.delete('activityType');
        }
    }

    if (pushState) {
        // Add to browser history so back/forward buttons work
        window.history.pushState({}, '', url);
    } else {
        // Just update URL without adding to history (for initial load)
        window.history.replaceState({}, '', url);
    }
}

// Keyboard shortcuts
document.addEventListener('keydown', function (e) {
    // Ctrl/Cmd + N for new meal
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'meal');
    }

    // Ctrl/Cmd + E for new event
    if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'event');
    }

    // Ctrl/Cmd + D for new drink
    if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'drink');
    }

    // F5 or Ctrl/Cmd + R for refresh
    if (e.key === 'F5' || ((e.ctrlKey || e.metaKey) && e.key === 'r')) {
        e.preventDefault();
        window.eventSpreadsheet.refreshWithDefaultDates();
    }
});

// Global error handler
window.addEventListener('error', function (e) {
    console.error('Global error:', e.error);

    // Show user-friendly error message
    const errorDiv = document.createElement('div');
    errorDiv.className = 'alert alert-error';
    errorDiv.style.position = 'fixed';
    errorDiv.style.top = '20px';
    errorDiv.style.right = '20px';
    errorDiv.style.zIndex = '10000';
    errorDiv.style.maxWidth = '400px';
    errorDiv.innerHTML = `
        <strong>Error:</strong> Something went wrong. Please refresh the page.
        <button onclick="this.parentElement.remove()" style="float: right; background: none; border: none; font-size: 1.2em; cursor: pointer;">&times;</button>
    `;

    document.body.appendChild(errorDiv);

    // Auto-remove after 5 seconds
    setTimeout(() => {
        if (errorDiv.parentElement) {
            errorDiv.remove();
        }
    }, 5000);
});

// Service worker registration for offline capabilities (future enhancement)
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/static/sw.js').catch(e => {
        console.log('Service worker registration failed:', e);
    });
}

// Utility functions
window.utils = {
    formatDate: function (dateString) {
        return new Date(dateString).toLocaleDateString();
    },

    formatPeople: function (people) {
        if (!people) return '';

        // Handle string format (comma-separated names) or array of objects
        let nameArray;
        if (typeof people === 'string') {
            nameArray = people.split(',').map(name => name.trim()).filter(name => name);
        } else if (Array.isArray(people)) {
            if (people.length === 0) return '';
            nameArray = people.map(p => p.name);
        } else {
            return '';
        }

        // Check if it's exactly just xx and ww - if so, omit them
        const lowerNames = nameArray.map(name => name.toLowerCase());
        if (lowerNames.length === 2 && lowerNames.includes('xx') && lowerNames.includes('ww')) {
            return '';
        }

        // Sort people: xx and ww first, then others alphabetically
        const sortedNames = nameArray.slice().sort((a, b) => {
            const aName = a.toLowerCase();
            const bName = b.toLowerCase();

            // xx comes first
            if (aName === 'xx') return -1;
            if (bName === 'xx') return 1;

            // ww comes second
            if (aName === 'ww') return -1;
            if (bName === 'ww') return 1;

            // Others alphabetically
            return aName.localeCompare(bName);
        });

        return sortedNames.join(', ');
    },

    debounce: function (func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    },

    showToast: function (message, type = 'info') {
        const toast = document.createElement('div');
        toast.className = `alert alert-${type}`;
        toast.style.position = 'fixed';
        toast.style.top = '20px';
        toast.style.right = '20px';
        toast.style.zIndex = '10000';
        toast.style.maxWidth = '400px';
        toast.innerHTML = `
            ${message}
            <button onclick="this.parentElement.remove()" style="float: right; background: none; border: none; font-size: 1.2em; cursor: pointer;">&times;</button>
        `;

        document.body.appendChild(toast);

        // Auto-remove after 3 seconds
        setTimeout(() => {
            if (toast.parentElement) {
                toast.remove();
            }
        }, 3000);
    }
};

console.log('XNote frontend initialized successfully!');