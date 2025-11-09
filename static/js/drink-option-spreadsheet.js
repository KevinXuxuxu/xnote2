/**
 * Drink Option Spreadsheet Component using Handsontable
 */
class DrinkOptionSpreadsheet {
    constructor(containerId) {
        this.containerId = containerId;
        this.hotInstance = null;
        this.data = [];
        this.filteredData = [];
        this.currentSearchText = '';

        this.initializeSpreadsheet();
    }

    initializeSpreadsheet() {
        const container = document.getElementById(this.containerId);

        const config = {
            data: [],
            licenseKey: 'non-commercial-and-evaluation',
            height: window.innerHeight - 165,
            width: '100%',
            colHeaders: ['Name'],
            columns: [
                {
                    data: 'name',
                    type: 'text',
                    width: 400,
                    validator: this.requiredValidator
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
            
            // Double-click detected - show error message
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
     * Handle double-click on a cell - show error message
     */
    handleCellDoubleClick(row, col) {
        alert('Drink options cannot be edited. To modify a drink option, delete the existing one and create a new one with the desired name.');
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

            const drinkOptions = await apiClient.getDrinkOptions();
            this.data = drinkOptions;
            this.applySearchFilter();

            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load drink options:', error);
            this.showError('Failed to load drink options from server');
            this.showLoading(false);
        }
    }

    /**
     * Handle cell changes - update drink option data
     */
    async onCellChange(changes, source) {
        if (source === 'loadData') return;

        for (const change of changes) {
            const [row, prop, oldValue, newValue] = change;

            if (oldValue === newValue) continue;

            const rowData = this.hotInstance.getDataAtRow(row);
            const drinkOptionName = rowData[0]; // Name is in first column

            if (!drinkOptionName || !oldValue) {
                // Skip new rows or rows without original name
                continue;
            }

            try {
                // For drink options, we need to delete the old one and create a new one
                // since the name is the primary key
                await apiClient.deleteDrinkOption(oldValue);
                await apiClient.createDrinkOption({ name: newValue });

            } catch (error) {
                console.error('Failed to update drink option:', error);
                this.showError(`Failed to update drink option: ${error.message}`);

                // Revert the change
                this.hotInstance.setDataAtCell(row, this.hotInstance.propToCol(prop), oldValue);
            }
        }
    }

    /**
     * Capture drink option data before row removal and show confirmation
     */
    beforeRowRemove(index, amount, physicalRows, source) {
        // Store the drink options that will be deleted
        this.drinkOptionsToDelete = [];

        for (const physicalRow of physicalRows) {
            // Get the actual displayed row data at the time of deletion
            const rowData = this.hotInstance.getDataAtRow(physicalRow);

            if (rowData && rowData[0]) { // Name is in column 0
                const drinkOptionName = rowData[0];

                this.drinkOptionsToDelete.push({
                    name: drinkOptionName,
                    physicalRow: physicalRow
                });
            }
        }

        // Show confirmation dialog
        if (this.drinkOptionsToDelete.length > 0) {
            const names = this.drinkOptionsToDelete.map(drinkOption => `"${drinkOption.name}"`).join(', ');
            const pluralText = this.drinkOptionsToDelete.length === 1 ? 'drink option' : 'drink options';
            const confirmed = confirm(
                `Are you sure you want to delete ${this.drinkOptionsToDelete.length} ${pluralText}: ${names}?\n\n` +
                `This will also delete all related drinks that use ${this.drinkOptionsToDelete.length === 1 ? 'this drink option' : 'these drink options'}.`
            );

            if (!confirmed) {
                // Cancel the deletion by clearing the list
                this.drinkOptionsToDelete = [];
                return false;
            }
        }
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.drinkOptionsToDelete || this.drinkOptionsToDelete.length === 0) {
            // User cancelled or no drink options to delete, refresh to restore rows
            await this.loadData();
            return;
        }

        // Delete each drink option from the backend
        for (const drinkOption of this.drinkOptionsToDelete) {
            try {
                await apiClient.deleteDrinkOption(drinkOption.name);
            } catch (error) {
                console.error('Failed to delete drink option:', error);
                this.showError(`Failed to delete drink option "${drinkOption.name}": ${error.message}`);
                // Refresh data to restore the state
                await this.loadData();
                return;
            }
        }

        // Clear the temporary storage
        this.drinkOptionsToDelete = [];

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
            this.filteredData = this.data.filter(drinkOption => 
                this.drinkOptionMatchesSearch(drinkOption, this.currentSearchText)
            );
        }
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Check if drink option matches search criteria
     */
    drinkOptionMatchesSearch(drinkOption, searchText) {
        return (
            (drinkOption.name && drinkOption.name.toLowerCase().includes(searchText))
        );
    }
}

// Make DrinkOptionSpreadsheet globally available
window.DrinkOptionSpreadsheet = DrinkOptionSpreadsheet;