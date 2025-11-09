/**
 * People Spreadsheet Component using Handsontable
 */
class PeopleSpreadsheet {
    constructor(containerId) {
        this.containerId = containerId;
        this.hotInstance = null;
        this.data = [];
        this.filteredData = [];
        this.currentSearchText = '';
        this.lastClickInfo = null;

        this.initializeSpreadsheet();
    }

    initializeSpreadsheet() {
        const container = document.getElementById(this.containerId);

        const config = {
            data: [],
            licenseKey: 'non-commercial-and-evaluation',
            height: window.innerHeight - 165,
            width: '100%',
            colHeaders: ['ID', 'Name', 'Notes'],
            columns: [
                {
                    data: 'id',
                    type: 'numeric',
                    readOnly: true,
                    width: 80
                },
                {
                    data: 'name',
                    type: 'text',
                    width: 200,
                    validator: this.requiredValidator
                },
                {
                    data: 'notes',
                    type: 'text',
                    width: 400
                }
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
                    'copy': {
                        name: 'Copy'
                    },
                    'cut': {
                        name: 'Cut'
                    }
                }
            },
            afterChange: this.onCellChange.bind(this),
            beforeRemoveRow: this.beforeRowRemove.bind(this),
            afterRemoveRow: this.afterRowRemove.bind(this),
            afterOnCellMouseUp: this.onCellMouseUp.bind(this)
        };

        this.hotInstance = new Handsontable(container, config);
        this.loadData();
    }

    /**
     * Handle mouse up events for double-click detection
     */
    onCellMouseUp(event, coords) {
        if (!coords) return;

        const row = coords.row;
        const col = coords.col;
        
        // Check if this is a double-click
        const now = Date.now();
        if (this.lastClickInfo && 
            this.lastClickInfo.row === row && 
            this.lastClickInfo.col === col &&
            (now - this.lastClickInfo.time) < 300) {
            
            // Double-click detected
            this.handleCellDoubleClick(row, col);
            this.lastClickInfo = null;
        } else {
            // Single-click - store for potential double-click
            this.lastClickInfo = {
                row: row,
                col: col,
                time: now
            };
        }
    }

    /**
     * Handle double-click on a cell
     */
    handleCellDoubleClick(row, col) {
        const rowData = this.hotInstance.getDataAtRow(row);
        
        if (rowData && rowData[0]) { // Check if person has an ID
            const personId = rowData[0];
            const personName = rowData[1];
            
            // Open the people form in edit mode
            if (window.peopleForm) {
                window.peopleForm.openModalForEdit(personId);
            }
        }
    }

    /**
     * Required field validator
     */
    requiredValidator(value, callback) {
        if (value === null || value === undefined || value === '') {
            callback(false);
        } else {
            callback(true);
        }
    }

    /**
     * Load data from API and populate spreadsheet
     */
    async loadData() {
        try {
            this.showLoading(true);

            const people = await apiClient.getPeople();
            this.data = people;
            this.applySearchFilter();

            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load people:', error);
            this.showError('Failed to load people from server');
            this.showLoading(false);
        }
    }

    /**
     * Handle cell changes - update person data
     */
    async onCellChange(changes, source) {
        if (source === 'loadData') return;

        for (const change of changes) {
            const [row, prop, oldValue, newValue] = change;

            if (oldValue === newValue) continue;

            const rowData = this.hotInstance.getDataAtRow(row);
            const personId = rowData[0]; // ID is in first column

            if (!personId) {
                // Skip rows without ID (shouldn't happen with modal approach)
                continue;
            }

            try {
                // Prepare update data
                const updateData = {};
                updateData[prop] = newValue;

                await apiClient.updatePerson(personId, updateData);

            } catch (error) {
                console.error('Failed to update person:', error);
                this.showError(`Failed to update person: ${error.message}`);

                // Revert the change
                this.hotInstance.setDataAtCell(row, this.hotInstance.propToCol(prop), oldValue);
            }
        }
    }

    /**
     * Capture person data before row removal and show confirmation
     */
    beforeRowRemove(index, amount, physicalRows, source) {
        // Store the people that will be deleted
        this.peopleToDelete = [];

        for (const physicalRow of physicalRows) {
            // Get the actual displayed row data at the time of deletion
            const rowData = this.hotInstance.getDataAtRow(physicalRow);

            if (rowData && rowData[0]) { // ID is in column 0
                const personId = rowData[0];
                const personName = rowData[1]; // Name is in column 1

                this.peopleToDelete.push({
                    id: personId,
                    name: personName,
                    physicalRow: physicalRow
                });
            }
        }

        // Show confirmation dialog
        if (this.peopleToDelete.length > 0) {
            const names = this.peopleToDelete.map(person => `"${person.name}"`).join(', ');
            const pluralText = this.peopleToDelete.length === 1 ? 'person' : 'people';
            const confirmed = confirm(
                `Are you sure you want to delete ${this.peopleToDelete.length} ${pluralText}: ${names}?\n\n` +
                `This will also delete all related meals, events, and drinks associated with ${this.peopleToDelete.length === 1 ? 'this person' : 'these people'}.`
            );

            if (!confirmed) {
                // Cancel the deletion by clearing the list
                this.peopleToDelete = [];
                return false; // This should prevent the deletion, but Handsontable might not respect it
            }
        }
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.peopleToDelete || this.peopleToDelete.length === 0) {
            // User cancelled or no people to delete, refresh to restore rows
            await this.loadData();
            return;
        }

        // Delete each person from the backend
        for (const person of this.peopleToDelete) {
            try {
                await apiClient.deletePerson(person.id);
            } catch (error) {
                console.error('Failed to delete person:', error);
                this.showError(`Failed to delete person "${person.name}": ${error.message}`);
                // Refresh data to restore the state
                await this.loadData();
                return;
            }
        }

        // Clear the temporary storage
        this.peopleToDelete = [];

        // Refresh data to reflect the changes
        await this.loadData();
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
        alert(`Error: ${message}`);
    }

    /**
     * Set search filter text and apply it
     */
    setSearchFilter(searchText) {
        this.currentSearchText = searchText.toLowerCase();
        this.applySearchFilter();
    }

    /**
     * Apply search filter to the data
     */
    applySearchFilter() {
        if (!this.currentSearchText.trim()) {
            // No search filter, show all data
            this.filteredData = [...this.data];
        } else {
            // Filter data based on search text
            this.filteredData = this.data.filter(person => 
                this.personMatchesSearch(person, this.currentSearchText)
            );
        }
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Check if person matches search criteria
     */
    personMatchesSearch(person, searchText) {
        return (
            (person.name && person.name.toLowerCase().includes(searchText))
        );
    }
}

// Make PeopleSpreadsheet globally available
window.PeopleSpreadsheet = PeopleSpreadsheet;