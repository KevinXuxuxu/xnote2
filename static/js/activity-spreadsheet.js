/**
 * Activity Spreadsheet Component using Handsontable
 */
class ActivitySpreadsheet {
    constructor(containerId) {
        this.containerId = containerId;
        this.hotInstance = null;
        this.data = [];
        
        this.initializeSpreadsheet();
    }

    initializeSpreadsheet() {
        const container = document.getElementById(this.containerId);
        
        const config = {
            data: [],
            licenseKey: 'non-commercial-and-evaluation',
            height: window.innerHeight - 150,
            width: '100%',
            colHeaders: ['ID', 'Name', 'Activity Type'],
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
                    width: 300,
                    validator: this.requiredValidator
                },
                { 
                    data: 'type',
                    type: 'text',
                    width: 200,
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
            afterRemoveRow: this.afterRowRemove.bind(this)
        };

        this.hotInstance = new Handsontable(container, config);
        this.loadData();
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
            
            const activities = await apiClient.getActivities();
            this.data = activities;
            this.hotInstance.loadData(this.data);
            
            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load activities:', error);
            this.showError('Failed to load activities from server');
            this.showLoading(false);
        }
    }

    /**
     * Handle cell changes - update activity data
     */
    async onCellChange(changes, source) {
        if (source === 'loadData') return;
        
        for (const change of changes) {
            const [row, prop, oldValue, newValue] = change;
            
            if (oldValue === newValue) continue;
            
            const rowData = this.hotInstance.getDataAtRow(row);
            const activityId = rowData[0]; // ID is in first column
            
            if (!activityId) {
                // Skip rows without ID (shouldn't happen with modal approach)
                continue;
            }
            
            try {
                // Prepare update data
                const updateData = {};
                updateData[prop] = newValue;
                
                await apiClient.updateActivity(activityId, updateData);
                
            } catch (error) {
                console.error('Failed to update activity:', error);
                this.showError(`Failed to update activity: ${error.message}`);
                
                // Revert the change
                this.hotInstance.setDataAtCell(row, this.hotInstance.propToCol(prop), oldValue);
            }
        }
    }

    /**
     * Capture activity data before row removal
     */
    beforeRowRemove(index, amount, physicalRows, source) {
        // Store the activities that will be deleted
        this.activitiesToDelete = [];
        
        for (const physicalRow of physicalRows) {
            // Get the actual displayed row data at the time of deletion
            const rowData = this.hotInstance.getDataAtRow(physicalRow);
            
            if (rowData && rowData[0]) { // ID is in column 0
                const activityId = rowData[0];
                const activityName = rowData[1]; // Name is in column 1
                
                this.activitiesToDelete.push({
                    id: activityId,
                    name: activityName,
                    physicalRow: physicalRow
                });
            }
        }
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.activitiesToDelete || this.activitiesToDelete.length === 0) {
            return;
        }
        
        // Delete each activity from the backend
        for (const activity of this.activitiesToDelete) {
            try {
                await apiClient.deleteActivity(activity.id);
            } catch (error) {
                console.error('Failed to delete activity:', error);
                this.showError(`Failed to delete activity "${activity.name}": ${error.message}`);
                // Refresh data to restore the state
                await this.loadData();
                return;
            }
        }
        
        // Clear the temporary storage
        this.activitiesToDelete = [];
        
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
}

// Make ActivitySpreadsheet globally available
window.ActivitySpreadsheet = ActivitySpreadsheet;