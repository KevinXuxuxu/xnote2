/**
 * Main Spreadsheet Component using Handsontable
 */
class EventSpreadsheet {
    constructor(containerId) {
        this.containerId = containerId;
        this.hotInstance = null;
        this.data = [];
        this.filteredData = [];

        // Initialize filters from URL parameters if available
        const urlParams = new URLSearchParams(window.location.search);
        this.currentFilters = {
            startDate: urlParams.get('startDate') || null,
            endDate: urlParams.get('endDate') || null,
            searchText: urlParams.get('searchText') || ''
        };

        // Track column visibility state
        this.mealsAndDrinksHidden = false;

        this.initializeSpreadsheet();
    }

    initializeSpreadsheet() {
        const container = document.getElementById(this.containerId);

        const config = {
            data: [],
            licenseKey: 'non-commercial-and-evaluation',
            height: window.innerHeight - 210,
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
                    width: 100,
                    renderer: this.dateRenderer.bind(this)
                },
                {
                    data: 'day_of_week',
                    type: 'text',
                    readOnly: true,
                    width: 45,
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
                    width: 100,
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
                    width: 100,
                    renderer: this.mealRenderer.bind(this)
                },
                {
                    data: 'drinks',
                    type: 'text',
                    readOnly: true,
                    width: 110,
                    renderer: this.drinkRenderer.bind(this)
                },
                ...Array.from({ length: 10 }, (_, i) => ({
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
            hiddenColumns: {
                copyPasteEnabled: true,
                indicators: true,
                columns: []
            },
            contextMenu: {
                items: {
                    'details': {
                        name: 'View/Edit Details',
                        callback: (key, selection) => {
                            const row = selection[0].start.row;
                            this.openDetails(row);
                        }
                    },
                    'sp1': '---------',
                    'delete_event': {
                        name: 'Delete Event',
                        hidden: () => {
                            const selection = this.hotInstance.getSelected();
                            if (!selection || !selection[0]) return true;

                            const row = selection[0][0];
                            const col = selection[0][1];

                            // Hide if not an event cell (columns 9-18)
                            if (col < 9 || col > 18) return true;

                            const rowData = this.filteredData[row];
                            const eventIndex = col - 9;

                            // Hide if no event in this cell
                            return !(rowData && rowData.events && rowData.events[eventIndex] && rowData.events[eventIndex].id);
                        },
                        callback: (key, selection) => {
                            const row = selection[0].start.row;
                            const col = selection[0].start.col;
                            this.deleteEvent(row, col);
                        }
                    },
                    'sp2': '---------',
                    'delete_meal': {
                        name: 'Delete Meal',
                        hidden: () => {
                            const selection = this.hotInstance.getSelected();
                            if (!selection || !selection[0]) return true;

                            const row = selection[0][0];
                            const col = selection[0][1];

                            // Hide if not a meal cell (columns 2-7: breakfast.0, breakfast.1, lunch.0, lunch.1, dinner.0, dinner.1)
                            if (col < 2 || col > 7) return true;

                            const rowData = this.filteredData[row];

                            // Determine meal time and index
                            let mealTime, mealIndex;
                            if (col === 2 || col === 3) {
                                mealTime = 'breakfast';
                                mealIndex = col - 2;
                            } else if (col === 4 || col === 5) {
                                mealTime = 'lunch';
                                mealIndex = col - 4;
                            } else {
                                mealTime = 'dinner';
                                mealIndex = col - 6;
                            }

                            // Hide if no meal in this cell
                            return !(rowData && rowData[mealTime] && rowData[mealTime][mealIndex] &&
                                rowData[mealTime][mealIndex].ids && rowData[mealTime][mealIndex].ids.length > 0);
                        },
                        callback: (key, selection) => {
                            const row = selection[0].start.row;
                            const col = selection[0].start.col;
                            this.deleteMeal(row, col);
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

            // Build display text from structured data
            let displayText = '';

            // Add people if present (using formatPeople utility for consistent ordering)
            const formattedPeople = window.utils.formatPeople(mealItem.people);
            if (formattedPeople) {
                displayText += formattedPeople + ' ';
            }

            // Add food name
            displayText += mealItem.name || '';

            // Add notes if present
            if (mealItem.notes && mealItem.notes.trim()) {
                displayText += ' (' + mealItem.notes + ')';
            }

            td.textContent = displayText;

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

            // Post-process data to merge meals with same criteria
            this.data = this.postProcessMealMerging(dailySummaries);
            this.applyFilters();

            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load data:', error);
            this.showError('Failed to load data from server');
            this.showLoading(false);
        }
    }

    getDefaultStartDate() {
        return window.dateUtils.getDaysAgoLocal(30);
    }

    getDefaultEndDate() {
        return window.dateUtils.getTodayLocal();
    }

    /**
     * Post-process daily summaries to merge meals with same criteria
     * Merges meals with same date, time, meal_type, and source type
     */
    postProcessMealMerging(dailySummaries) {
        return dailySummaries.map(day => {
            const processedDay = { ...day };

            // Process each meal time (breakfast, lunch, dinner)
            ['breakfast', 'lunch', 'dinner'].forEach(mealTime => {
                if (day[mealTime] && day[mealTime].length > 0) {
                    processedDay[mealTime] = this.mergeMealsByType(day[mealTime]);
                }
            });

            return processedDay;
        });
    }

    /**
     * Merge meals of the same type and source
     * Groups by meal_type AND all other attributes (people, notes) and merges names with commas
     */
    mergeMealsByType(meals) {
        // Group meals by ALL attributes except the food name
        const grouped = meals.reduce((groups, meal) => {
            // Create a key that includes everything except the food name
            const key = `${meal.type || 'unknown'}|${meal.people || ''}|${meal.notes || ''}`;

            if (!groups[key]) {
                groups[key] = [];
            }
            groups[key].push(meal);
            return groups;
        }, {});

        // Merge meals within each group
        const merged = [];

        Object.keys(grouped).forEach(groupKey => {
            const mealsInGroup = grouped[groupKey];

            if (mealsInGroup.length === 1) {
                // Single meal, no merging needed
                merged.push(mealsInGroup[0]);
            } else {
                // Multiple meals with identical attributes, merge their food names
                const mergedMeal = this.mergeMealGroup(mealsInGroup);
                merged.push(mergedMeal);
            }
        });

        // Sort by display string length (longer first)
        merged.sort((a, b) => {
            const aDisplayLength = this.getMealDisplayLength(a);
            const bDisplayLength = this.getMealDisplayLength(b);
            return bDisplayLength - aDisplayLength;
        });

        return merged;
    }

    /**
     * Calculate display string length for a meal (used for sorting)
     */
    getMealDisplayLength(meal) {
        let displayText = '';

        // Add people if present
        const formattedPeople = window.utils.formatPeople(meal.people);
        if (formattedPeople) {
            displayText += formattedPeople + ' ';
        }

        // Add food name
        displayText += meal.name || '';

        // Add notes if present
        if (meal.notes && meal.notes.trim()) {
            displayText += ' (' + meal.notes + ')';
        }

        return displayText.length;
    }

    /**
     * Merge a group of meals with identical attributes (people, notes, type)
     */
    mergeMealGroup(meals) {
        // All meals in this group have identical people, notes, and type
        // Just extract food names and combine them
        const foodNames = meals.map(meal => meal.name).filter(name => name.length > 0);

        // Combine all IDs from all meals in the group
        const allIds = meals.reduce((ids, meal) => {
            if (meal.ids && Array.isArray(meal.ids)) {
                ids.push(...meal.ids);
            }
            return ids;
        }, []);

        // Use the first meal as template since all attributes should be identical
        const template = meals[0];

        return {
            ids: allIds,                 // Combined IDs from all meals in group
            name: foodNames.join(', '),  // Combine food names with commas
            people: template.people,     // Same people for all in group
            notes: template.notes,       // Same notes for all in group
            type: template.type          // Same type for all in group
        };
    }



    /**
     * Apply current filters to data
     */
    applyFilters() {
        // Date range filtering is handled by the backend API
        // Apply client-side text search filtering
        this.filteredData = [...this.data];

        if (this.currentFilters.searchText && this.currentFilters.searchText.trim()) {
            const searchTerm = this.currentFilters.searchText.toLowerCase().trim();
            this.filteredData = this.filteredData.filter(row => this.rowMatchesSearch(row, searchTerm));
        }

        this.hotInstance.loadData(this.filteredData);

        // Reapply column hiding if columns were previously hidden
        if (this.mealsAndDrinksHidden) {
            // Add small delay to ensure Handsontable is fully rendered
            setTimeout(() => {
                this.reapplyColumnHiding();
            }, 0);
        }
    }

    /**
     * Check if a row matches the search term by checking all displayable content
     */
    rowMatchesSearch(row, searchTerm) {
        // Check date and day
        if (row.date && row.date.toLowerCase().includes(searchTerm)) return true;
        if (row.day_of_week && row.day_of_week.toLowerCase().includes(searchTerm)) return true;

        // Check meals (breakfast, lunch, dinner)
        for (const mealTime of ['breakfast', 'lunch', 'dinner']) {
            if (row[mealTime] && Array.isArray(row[mealTime])) {
                for (const meal of row[mealTime]) {
                    const displayText = this.getMealDisplayText(meal).toLowerCase();
                    if (displayText.includes(searchTerm)) return true;
                }
            }
        }

        // Check drinks
        if (row.drinks && Array.isArray(row.drinks)) {
            for (const drink of row.drinks) {
                if (drink.toLowerCase().includes(searchTerm)) return true;
            }
        }

        // Check events
        if (row.events && Array.isArray(row.events)) {
            for (const event of row.events) {
                if (event.text && event.text.toLowerCase().includes(searchTerm)) return true;
            }
        }

        return false;
    }

    /**
     * Get display text for a meal (same logic as mealRenderer)
     */
    getMealDisplayText(meal) {
        let displayText = '';

        // Add people if present
        const formattedPeople = window.utils.formatPeople(meal.people);
        if (formattedPeople) {
            displayText += formattedPeople + ' ';
        }

        // Add food name
        displayText += meal.name || '';

        // Add notes if present
        if (meal.notes && meal.notes.trim()) {
            displayText += ' (' + meal.notes + ')';
        }

        return displayText;
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
     * Delete an event from the selected cell
     */
    async deleteEvent(row, col) {
        const rowData = this.filteredData[row];
        const eventIndex = col - 9; // Convert column to event index

        if (!rowData || !rowData.events || !rowData.events[eventIndex]) {
            this.showError('No event found in this cell');
            return;
        }

        const event = rowData.events[eventIndex];
        if (!event.id) {
            this.showError('Cannot delete event: no ID found');
            return;
        }

        // Show confirmation dialog
        const eventText = event.text || 'this event';
        const confirmed = confirm(`Are you sure you want to delete "${eventText}"?`);

        if (!confirmed) {
            return;
        }

        try {
            this.showLoading(true);

            // Delete event via API
            await apiClient.deleteEvent(event.id);

            // Refresh data to reflect the changes
            await this.loadData();

            this.showLoading(false);

            // Show success message
            window.utils.showToast(`Event "${eventText}" deleted successfully`, 'success');

        } catch (error) {
            console.error('Failed to delete event:', error);
            this.showError(`Failed to delete event: ${error.message}`);
            this.showLoading(false);
        }
    }

    /**
     * Delete meals from the selected cell
     */
    async deleteMeal(row, col) {
        const rowData = this.filteredData[row];

        // Determine meal time and index
        let mealTime, mealIndex;
        if (col === 2 || col === 3) {
            mealTime = 'breakfast';
            mealIndex = col - 2;
        } else if (col === 4 || col === 5) {
            mealTime = 'lunch';
            mealIndex = col - 4;
        } else if (col === 6 || col === 7) {
            mealTime = 'dinner';
            mealIndex = col - 6;
        } else {
            this.showError('Invalid meal cell');
            return;
        }

        if (!rowData || !rowData[mealTime] || !rowData[mealTime][mealIndex]) {
            this.showError('No meal found in this cell');
            return;
        }

        const meal = rowData[mealTime][mealIndex];
        if (!meal.ids || meal.ids.length === 0) {
            this.showError('Cannot delete meal: no IDs found');
            return;
        }

        // Show confirmation dialog with count
        const mealText = meal.name || 'this meal';
        const mealCount = meal.ids.length;
        const pluralText = mealCount === 1 ? 'meal' : 'meals';
        const confirmed = confirm(`Are you sure you want to delete ${mealCount} ${pluralText} in "${mealText}"?`);

        if (!confirmed) {
            return;
        }

        try {
            this.showLoading(true);

            // Check if batch delete method is available
            if (typeof apiClient.deleteMealsBatch === 'function') {
                // Delete meals via batch API
                await apiClient.deleteMealsBatch(meal.ids);
            } else {
                console.warn('Batch delete not available, falling back to individual deletes');
                // Fallback: delete meals individually
                for (const mealId of meal.ids) {
                    await apiClient.deleteMeal(mealId);
                }
            }

            // Refresh data to reflect the changes
            await this.loadData();

            this.showLoading(false);

            // Show success message
            window.utils.showToast(`${mealCount} ${pluralText} deleted successfully`, 'success');

        } catch (error) {
            console.error('Failed to delete meals:', error);
            this.showError(`Failed to delete meals: ${error.message}`);
            this.showLoading(false);
        }
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
     * Reset to default date range and refresh data
     */
    async refreshWithDefaultDates() {
        // Reset filters to default date range but preserve search text
        this.currentFilters = {
            startDate: this.getDefaultStartDate(),
            endDate: this.getDefaultEndDate(),
            searchText: this.currentFilters.searchText || ''
        };

        // Update the date input fields in the UI
        document.getElementById('startDate').value = this.currentFilters.startDate;
        document.getElementById('endDate').value = this.currentFilters.endDate;

        // Update URL with new date range
        const updateUrlFilters = window.updateUrlFilters;
        if (typeof updateUrlFilters === 'function') {
            updateUrlFilters(this.currentFilters.startDate, this.currentFilters.endDate, this.currentFilters.searchText);
        }

        // Load data with new date range
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
     * Toggle visibility of meal and drink columns
     */
    toggleMealsAndDrinksColumns() {
        if (!this.hotInstance) return;

        // Columns to hide: Breakfast 1, Breakfast 2, Lunch 1, Lunch 2, Dinner 1, Dinner 2, Drinks
        // Column indices: 2, 3, 4, 5, 6, 7, 8
        const mealAndDrinkColumns = [2, 3, 4, 5, 6, 7, 8];

        // Access the hiddenColumns plugin instance
        const hiddenColumnsPlugin = this.hotInstance.getPlugin('hiddenColumns');

        if (this.mealsAndDrinksHidden) {
            // Show the columns
            hiddenColumnsPlugin.showColumns(mealAndDrinkColumns);
            this.mealsAndDrinksHidden = false;
        } else {
            // Hide the columns
            hiddenColumnsPlugin.hideColumns(mealAndDrinkColumns);
            this.mealsAndDrinksHidden = true;
        }

        // Re-render to see the changes
        this.hotInstance.render();
    }

    /**
     * Reapply column hiding after data reload (used internally)
     */
    reapplyColumnHiding() {
        if (!this.hotInstance) return;

        // Columns to hide: Breakfast 1, Breakfast 2, Lunch 1, Lunch 2, Dinner 1, Dinner 2, Drinks
        // Column indices: 2, 3, 4, 5, 6, 7, 8
        const mealAndDrinkColumns = [2, 3, 4, 5, 6, 7, 8];

        // Access the hiddenColumns plugin instance
        const hiddenColumnsPlugin = this.hotInstance.getPlugin('hiddenColumns');
        
        // Hide the columns (we know they should be hidden because mealsAndDrinksHidden is true)
        hiddenColumnsPlugin.hideColumns(mealAndDrinkColumns);

        // Re-render to see the changes
        this.hotInstance.render();
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