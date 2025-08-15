/**
 * Main Spreadsheet Component using Handsontable
 */
class EventSpreadsheet {
    constructor(containerId) {
        this.containerId = containerId;
        this.hotInstance = null;
        this.data = [];
        this.filteredData = [];
        this.currentFilters = {
            startDate: null,
            endDate: null
        };
        
        this.initializeSpreadsheet();
    }

    initializeSpreadsheet() {
        const container = document.getElementById(this.containerId);
        
        const config = {
            data: [],
            licenseKey: 'non-commercial-and-evaluation',
            height: 600,
            width: '100%',
            colHeaders: [
                'Date',
                'Day',
                'Breakfast 1',
                'Breakfast 2', 
                'Lunch 1',
                'Lunch 2',
                'Dinner 1',
                'Dinner 2',
                'Drinks',
                'Event 1',
                'Event 2',
                'Event 3',
                'Event 4',
                'Event 5',
                'Event 6',
                'Event 7',
                'Event 8',
                'Event 9',
                'Event 10'
            ],
            columns: [
                { 
                    data: 'date',
                    type: 'date',
                    dateFormat: 'YYYY-MM-DD',
                    readOnly: true,
                    width: 110,
                    renderer: this.dateRenderer.bind(this)
                },
                { 
                    data: 'day_of_week',
                    type: 'text',
                    readOnly: true,
                    width: 50,
                    renderer: this.dayRenderer.bind(this)
                },
                { 
                    data: 'breakfast.0',
                    type: 'text',
                    readOnly: true,
                    width: 150,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'breakfast.1',
                    type: 'text',
                    readOnly: true,
                    width: 100,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'lunch.0',
                    type: 'text',
                    readOnly: true,
                    width: 200,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'lunch.1',
                    type: 'text',
                    readOnly: true,
                    width: 200,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'dinner.0',
                    type: 'text',
                    readOnly: true,
                    width: 200,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'dinner.1',
                    type: 'text',
                    readOnly: true,
                    width: 200,
                    renderer: this.mealRenderer.bind(this)
                },
                { 
                    data: 'drinks',
                    type: 'text',
                    readOnly: true,
                    width: 150,
                    renderer: this.drinkRenderer.bind(this)
                },
                ...Array.from({length: 10}, (_, i) => ({
                    data: `events.${i}`,
                    type: 'text',
                    readOnly: true,
                    width: 200,
                    renderer: this.eventRenderer.bind(this)
                }))
            ],
            rowHeaders: false,
            columnSorting: true,
            filters: true,
            dropdownMenu: true,
            contextMenu: {
                items: {
                    'row_above': {
                        name: 'Insert row above'
                    },
                    'row_below': {
                        name: 'Insert row below'  
                    },
                    'remove_row': {
                        name: 'Delete row'
                    },
                    'hsep1': '---------',
                    'details': {
                        name: 'View/Edit Details',
                        callback: (key, selection) => {
                            const row = selection[0].start.row;
                            this.openDetails(row);
                        }
                    }
                }
            },
            afterChange: this.onCellChange.bind(this),
            afterRemoveRow: this.onRowRemove.bind(this),
            beforeCreateRow: this.onRowCreate.bind(this)
        };

        this.hotInstance = new Handsontable(container, config);
        this.loadData();
    }

    /**
     * Custom renderer for date cells with today/weekend highlighting
     */
    dateRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.DateRenderer.apply(this, arguments);
        
        if (!value) return;
        
        const date = new Date(value);
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        date.setHours(0, 0, 0, 0);
        
        // Clear existing classes
        td.className = td.className || '';
        td.className = td.className.replace(/date-(today|weekend)/g, '');
        
        // Check if it's today
        if (date.getTime() === today.getTime()) {
            td.className += ' date-today';
        }
        // Check if it's weekend
        else {
            const dayOfWeek = date.getDay();
            if (dayOfWeek === 0) { // Sunday
                td.className += ' date-sunday';
            } else if (dayOfWeek === 6) { // Saturday
                td.className += ' date-saturday';
            }
        }
    }

    /**
     * Custom renderer for day cells with weekend highlighting
     */
    dayRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.TextRenderer.apply(this, arguments);
        
        // Clear existing classes
        td.className = td.className || '';
        td.className = td.className.replace(/day-(today|weekend)/g, '');
        
        // Get the corresponding date from the same row
        const rowData = this.filteredData[row];
        if (!rowData || !rowData.date) return;
        
        const date = new Date(rowData.date);
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        date.setHours(0, 0, 0, 0);
        
        // Check if it's today
        if (date.getTime() === today.getTime()) {
            td.className += ' day-today';
        }
        // Check if it's weekend
        else {
            const dayOfWeek = date.getDay();
            if (dayOfWeek === 0) { // Sunday
                td.className += ' day-sunday';
            } else if (dayOfWeek === 6) { // Saturday
                td.className += ' day-saturday';
            }
        }
    }

    /**
     * Custom renderer for meal cells with meal type coloring
     */
    mealRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.TextRenderer.apply(this, arguments);
        
        // Clear existing meal type classes
        td.className = td.className || '';
        td.className = td.className.replace(/meal-type-\w+/g, '');
        
        // Get the meal data from the row
        const rowData = this.filteredData[row];
        if (!rowData) return;
        
        // Extract meal time and index from prop (e.g., "breakfast.0")
        const [mealTime, indexStr] = prop.split('.');
        const index = parseInt(indexStr);
        
        if (rowData[mealTime] && rowData[mealTime][index]) {
            const mealItem = rowData[mealTime][index];
            
            // Display the text
            td.textContent = mealItem.text || '';
            
            // Add CSS class based on meal type
            if (mealItem.type) {
                td.className += ` meal-type-${mealItem.type}`;
            }
        } else {
            td.textContent = '';
        }
    }

    /**
     * Custom renderer for drink cells
     */
    drinkRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.TextRenderer.apply(this, arguments);
        
        td.className = td.className || '';
        td.className = td.className.replace(/event-type-\w+/g, '');
        
        if (value && Array.isArray(value) && value.length > 0) {
            td.textContent = value.join(', ');
            td.className += ' event-type-drink';
        }
    }

    /**
     * Custom renderer for event cells with activity type coloring
     */
    eventRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.TextRenderer.apply(this, arguments);
        
        // Clear existing activity type classes
        td.className = td.className || '';
        td.className = td.className.replace(/activity-type-\w+/g, '');
        td.className = td.className.replace(/event-type-\w+/g, '');
        
        // Get the event data from the row
        const rowData = this.filteredData[row];
        if (!rowData) return;
        
        // Extract event index from prop (e.g., "events.0")
        const [, indexStr] = prop.split('.');
        const index = parseInt(indexStr);
        
        if (rowData.events && rowData.events[index]) {
            const eventItem = rowData.events[index];
            
            // Display the text
            td.textContent = eventItem.text || '';
            
            // Add CSS class based on activity type
            if (eventItem.type) {
                // Normalize activity type for CSS class (replace spaces with hyphens)
                const normalizedType = eventItem.type.replace(/\s+/g, '-');
                td.className += ` activity-type-${normalizedType}`;
            }
        } else {
            td.textContent = '';
        }
    }

    /**
     * Load data from API and populate spreadsheet
     */
    async loadData() {
        try {
            this.showLoading(true);
            
            // Get date range from filters or use defaults
            const startDate = this.currentFilters.startDate || this.getDefaultStartDate();
            const endDate = this.currentFilters.endDate || this.getDefaultEndDate();
            
            const dailySummaries = await apiClient.getDailySummary(startDate, endDate);
            this.data = dailySummaries;
            this.applyFilters();
            
            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load data:', error);
            this.showError('Failed to load data from server');
            this.showLoading(false);
        }
    }

    getDefaultStartDate() {
        const date = new Date();
        date.setDate(date.getDate() - 30);
        return date.toISOString().split('T')[0];
    }

    getDefaultEndDate() {
        return new Date().toISOString().split('T')[0];
    }


    /**
     * Apply current filters to data
     */
    applyFilters() {
        // Date range filtering is now handled by the backend API
        // No additional client-side filtering needed for daily view
        this.filteredData = [...this.data];
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Set filters and refresh data
     */
    setFilters(filters) {
        const oldFilters = { ...this.currentFilters };
        this.currentFilters = { ...this.currentFilters, ...filters };
        
        // If date filters changed, reload data from API
        if (filters.startDate !== undefined || filters.endDate !== undefined) {
            this.loadData();
        } else {
            // Otherwise just apply client-side filters
            this.applyFilters();
        }
    }

    /**
     * Handle cell changes (disabled for daily view)
     */
    async onCellChange(changes, source) {
        // Daily view is read-only - no cell changes allowed
        return;
    }

    /**
     * Handle row removal (disabled for daily view)
     */
    async onRowRemove(index, amount, physicalRows) {
        // Daily view is read-only - no row removal allowed
        return;
    }

    /**
     * Handle row creation (disabled for daily view)
     */
    onRowCreate(index, amount) {
        // Daily view is read-only - no row creation allowed
        return;
    }

    /**
     * Open details modal for a row (redirects to daily detail view)
     */
    openDetails(row) {
        const rowData = this.filteredData[row];
        if (!rowData) return;
        
        // For daily view, we could open a day-specific detail modal
        // For now, just show the date-specific events
        alert(`View details for ${rowData.date} - ${rowData.day_of_week}`);
        // TODO: Implement daily detail view modal
    }

    /**
     * Refresh spreadsheet data
     */
    async refresh() {
        await this.loadData();
    }

    /**
     * Show loading state
     */
    showLoading(show) {
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) {
            overlay.style.display = show ? 'flex' : 'none';
        }
    }

    /**
     * Show error message
     */
    showError(message) {
        // Simple alert for now - could be enhanced with a toast system
        alert(`Error: ${message}`);
    }

    /**
     * Add new event of specified type (opens detail modal)
     */
    addNewRow(eventType) {
        // In daily view, adding new events opens the detail forms
        if (window.detailForms) {
            window.detailForms.openDetails(null, eventType);
        }
    }
}

// Make EventSpreadsheet globally available
window.EventSpreadsheet = EventSpreadsheet;