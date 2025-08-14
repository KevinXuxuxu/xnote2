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
            endDate: null,
            type: ''
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
                'ID',
                'Date', 
                'Type',
                'Time/Activity',
                'Summary',
                'People',
                'Location',
                'Notes'
            ],
            columns: [
                { 
                    data: 'id',
                    type: 'numeric',
                    readOnly: true,
                    width: 60
                },
                { 
                    data: 'date',
                    type: 'date',
                    dateFormat: 'YYYY-MM-DD',
                    width: 100
                },
                { 
                    data: 'eventType',
                    type: 'dropdown',
                    source: ['meal', 'event', 'drink'],
                    width: 80,
                    renderer: this.eventTypeRenderer.bind(this)
                },
                { 
                    data: 'timeOrActivity',
                    type: 'text',
                    width: 120
                },
                { 
                    data: 'summary',
                    type: 'text',
                    width: 200
                },
                { 
                    data: 'peopleNames',
                    type: 'text',
                    readOnly: true,
                    width: 150
                },
                { 
                    data: 'location',
                    type: 'text',
                    width: 120
                },
                { 
                    data: 'notes',
                    type: 'text',
                    width: 200
                }
            ],
            rowHeaders: true,
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
     * Custom renderer for event type column
     */
    eventTypeRenderer(instance, td, row, col, prop, value, cellProperties) {
        Handsontable.renderers.DropdownRenderer.apply(this, arguments);
        
        // Add CSS class based on event type
        td.className = td.className || '';
        td.className = td.className.replace(/event-type-\w+/g, '');
        
        if (value) {
            td.className += ` event-type-${value}`;
        }
    }

    /**
     * Load data from API and populate spreadsheet
     */
    async loadData() {
        try {
            this.showLoading(true);
            
            const events = await apiClient.getAllEvents();
            this.data = this.transformDataForSpreadsheet(events);
            this.applyFilters();
            
            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load data:', error);
            this.showError('Failed to load data from server');
            this.showLoading(false);
        }
    }

    /**
     * Transform API data for spreadsheet display
     */
    transformDataForSpreadsheet(events) {
        return events.map(event => {
            let timeOrActivity = '';
            let summary = '';
            let location = event.location || '';
            
            switch (event.eventType) {
                case 'meal':
                    timeOrActivity = event.time || '';
                    summary = this.getMealSummary(event);
                    break;
                case 'event':
                    timeOrActivity = event.activity ? `Activity ${event.activity}` : '';
                    summary = this.getEventSummary(event);
                    location = event.location || '';
                    break;
                case 'drink':
                    timeOrActivity = '';
                    summary = event.name || '';
                    break;
            }

            return {
                id: event.id,
                date: event.date,
                eventType: event.eventType,
                timeOrActivity,
                summary,
                peopleNames: '', // Will be populated when details are loaded
                location,
                notes: event.notes || '',
                _originalData: event
            };
        });
    }

    getMealSummary(meal) {
        // Simple summary for now - will be enhanced when we get meal details
        return 'Meal';
    }

    getEventSummary(event) {
        // Simple summary for now - will be enhanced when we get event details
        return `Activity ${event.activity}`;
    }

    /**
     * Apply current filters to data
     */
    applyFilters() {
        let filtered = [...this.data];
        
        // Date range filter
        if (this.currentFilters.startDate) {
            filtered = filtered.filter(row => 
                new Date(row.date) >= new Date(this.currentFilters.startDate)
            );
        }
        
        if (this.currentFilters.endDate) {
            filtered = filtered.filter(row => 
                new Date(row.date) <= new Date(this.currentFilters.endDate)
            );
        }
        
        // Type filter
        if (this.currentFilters.type) {
            filtered = filtered.filter(row => 
                row.eventType === this.currentFilters.type
            );
        }
        
        this.filteredData = filtered;
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Set filters and refresh data
     */
    setFilters(filters) {
        this.currentFilters = { ...this.currentFilters, ...filters };
        this.applyFilters();
    }

    /**
     * Handle cell changes
     */
    async onCellChange(changes, source) {
        if (!changes || source === 'loadData') return;
        
        for (const [row, prop, oldValue, newValue] of changes) {
            if (oldValue === newValue) continue;
            
            const rowData = this.filteredData[row];
            if (!rowData) continue;
            
            try {
                await this.updateRowData(rowData, prop, newValue);
            } catch (error) {
                console.error('Failed to update row:', error);
                this.showError('Failed to save changes');
                // Revert the change
                this.hotInstance.setDataAtCell(row, prop, oldValue, 'revert');
            }
        }
    }

    /**
     * Update row data via API
     */
    async updateRowData(rowData, property, newValue) {
        const { id, eventType, _originalData } = rowData;
        
        // Map spreadsheet properties to API properties
        let updateData = { ..._originalData };
        
        switch (property) {
            case 'date':
                updateData.date = newValue;
                break;
            case 'notes':
                updateData.notes = newValue;
                break;
            case 'location':
                if (eventType === 'event') {
                    updateData.location = newValue;
                }
                break;
            // Add more property mappings as needed
        }
        
        // Call appropriate API method
        switch (eventType) {
            case 'meal':
                await apiClient.updateMeal(id, updateData);
                break;
            case 'event':
                await apiClient.updateEvent(id, updateData);
                break;
            case 'drink':
                await apiClient.updateDrink(id, updateData);
                break;
        }
        
        // Update local data
        rowData._originalData = updateData;
    }

    /**
     * Handle row removal
     */
    async onRowRemove(index, amount, physicalRows) {
        for (const physicalRow of physicalRows) {
            const rowData = this.filteredData[physicalRow];
            if (!rowData) continue;
            
            try {
                await this.deleteRow(rowData);
            } catch (error) {
                console.error('Failed to delete row:', error);
                this.showError('Failed to delete row');
                // Reload data to revert changes
                this.loadData();
                return;
            }
        }
    }

    /**
     * Delete row via API
     */
    async deleteRow(rowData) {
        const { id, eventType } = rowData;
        
        switch (eventType) {
            case 'meal':
                await apiClient.deleteMeal(id);
                break;
            case 'event':
                await apiClient.deleteEvent(id);
                break;
            case 'drink':
                await apiClient.deleteDrink(id);
                break;
        }
    }

    /**
     * Handle row creation
     */
    onRowCreate(index, amount) {
        // Add empty rows with today's date
        const today = new Date().toISOString().split('T')[0];
        const newRows = Array(amount).fill().map(() => ({
            id: null,
            date: today,
            eventType: 'meal',
            timeOrActivity: '',
            summary: '',
            peopleNames: '',
            location: '',
            notes: '',
            _originalData: null
        }));
        
        this.filteredData.splice(index, 0, ...newRows);
    }

    /**
     * Open details modal for a row
     */
    openDetails(row) {
        const rowData = this.filteredData[row];
        if (!rowData) return;
        
        if (window.detailForms) {
            window.detailForms.openDetails(rowData.id, rowData.eventType);
        }
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
     * Add new row of specified type
     */
    addNewRow(eventType) {
        const today = new Date().toISOString().split('T')[0];
        const newRow = {
            id: null,
            date: today,
            eventType: eventType,
            timeOrActivity: '',
            summary: '',
            peopleNames: '',
            location: '',
            notes: '',
            _originalData: null
        };
        
        this.filteredData.unshift(newRow);
        this.hotInstance.loadData(this.filteredData);
        
        // Focus on the new row
        this.hotInstance.selectCell(0, 1); // Focus on date column
    }
}

// Make EventSpreadsheet globally available
window.EventSpreadsheet = EventSpreadsheet;