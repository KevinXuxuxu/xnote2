/**
 * Recipe Spreadsheet Component using Handsontable
 */
class RecipeSpreadsheet {
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
            colHeaders: ['ID', 'Name', 'Ingredients', 'Procedure', 'Cautions'],
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
                    data: 'ingredients',
                    type: 'text',
                    width: 400,
                    validator: this.requiredValidator
                },
                {
                    data: 'procedure',
                    type: 'text',
                    width: 500,
                    validator: this.requiredValidator
                },
                {
                    data: 'cautions',
                    type: 'text',
                    width: 300
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

            const recipes = await apiClient.getRecipes();
            this.data = recipes;
            this.applySearchFilter();

            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load recipes:', error);
            this.showError('Failed to load recipes from server');
            this.showLoading(false);
        }
    }

    /**
     * Set search filter and apply it
     */
    setSearchFilter(searchText) {
        this.currentSearchText = searchText;
        this.applySearchFilter();
    }

    /**
     * Apply current search filter to data
     */
    applySearchFilter() {
        this.filteredData = [...this.data];
        
        if (this.currentSearchText && this.currentSearchText.trim()) {
            const searchTerm = this.currentSearchText.toLowerCase().trim();
            this.filteredData = this.filteredData.filter(recipe => this.recipeMatchesSearch(recipe, searchTerm));
        }
        
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Check if a recipe matches the search term
     */
    recipeMatchesSearch(recipe, searchTerm) {
        // Search in name
        if (recipe.name && recipe.name.toLowerCase().includes(searchTerm)) return true;
        
        // Search in ingredients
        if (recipe.ingredients && recipe.ingredients.toLowerCase().includes(searchTerm)) return true;
        
        // Search in procedure
        if (recipe.procedure && recipe.procedure.toLowerCase().includes(searchTerm)) return true;
        
        // Search in cautions
        if (recipe.cautions && recipe.cautions.toLowerCase().includes(searchTerm)) return true;
        
        return false;
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
        
        if (rowData && rowData[0]) { // Check if recipe has an ID
            const recipeId = rowData[0];
            const recipeName = rowData[1];
            
            // Open the recipe form in edit mode
            if (window.recipeForm) {
                window.recipeForm.openModalForEdit(recipeId);
            }
        }
    }

    /**
     * Handle cell changes - update recipe data
     */
    async onCellChange(changes, source) {
        if (source === 'loadData') return;

        for (const change of changes) {
            const [row, prop, oldValue, newValue] = change;

            if (oldValue === newValue) continue;

            const rowData = this.hotInstance.getDataAtRow(row);
            const recipeId = rowData[0]; // ID is in first column

            if (!recipeId) {
                // Skip rows without ID (shouldn't happen with modal approach)
                continue;
            }

            try {
                // Prepare update data
                const updateData = {};
                updateData[prop] = newValue;

                await apiClient.updateRecipe(recipeId, updateData);

            } catch (error) {
                console.error('Failed to update recipe:', error);
                this.showError(`Failed to update recipe: ${error.message}`);

                // Revert the change
                this.hotInstance.setDataAtCell(row, this.hotInstance.propToCol(prop), oldValue);
            }
        }
    }

    /**
     * Capture recipe data before row removal and show confirmation
     */
    beforeRowRemove(index, amount, physicalRows, source) {
        // Store the recipes that will be deleted
        this.recipesToDelete = [];

        for (const physicalRow of physicalRows) {
            // Get the actual displayed row data at the time of deletion
            const rowData = this.hotInstance.getDataAtRow(physicalRow);

            if (rowData && rowData[0]) { // ID is in column 0
                const recipeId = rowData[0];
                const recipeName = rowData[1]; // Name is in column 1

                this.recipesToDelete.push({
                    id: recipeId,
                    name: recipeName,
                    physicalRow: physicalRow
                });
            }
        }

        // Show confirmation dialog
        if (this.recipesToDelete.length > 0) {
            const names = this.recipesToDelete.map(recipe => `"${recipe.name}"`).join(', ');
            const pluralText = this.recipesToDelete.length === 1 ? 'recipe' : 'recipes';
            const confirmed = confirm(
                `Are you sure you want to delete ${this.recipesToDelete.length} ${pluralText}: ${names}?\n\n` +
                `This will also delete all related meals that use ${this.recipesToDelete.length === 1 ? 'this recipe' : 'these recipes'}.`
            );

            if (!confirmed) {
                // Cancel the deletion by clearing the list
                this.recipesToDelete = [];
                return false;
            }
        }
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.recipesToDelete || this.recipesToDelete.length === 0) {
            // User cancelled or no recipes to delete, refresh to restore rows
            await this.loadData();
            return;
        }

        // Delete each recipe from the backend
        for (const recipe of this.recipesToDelete) {
            try {
                await apiClient.deleteRecipe(recipe.id);
            } catch (error) {
                console.error('Failed to delete recipe:', error);
                this.showError(`Failed to delete recipe "${recipe.name}": ${error.message}`);
                // Refresh data to restore the state
                await this.loadData();
                return;
            }
        }

        // Clear the temporary storage
        this.recipesToDelete = [];

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

// Make RecipeSpreadsheet globally available
window.RecipeSpreadsheet = RecipeSpreadsheet;