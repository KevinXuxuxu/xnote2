/**
 * Product Spreadsheet Component using Handsontable
 */
class ProductSpreadsheet {
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
            colHeaders: ['ID', 'Name'],
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

            const products = await apiClient.getProducts();
            this.data = products;
            this.applySearchFilter();

            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load products:', error);
            this.showError('Failed to load products from server');
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
            this.filteredData = this.filteredData.filter(product => this.productMatchesSearch(product, searchTerm));
        }
        
        this.hotInstance.loadData(this.filteredData);
    }

    /**
     * Check if a product matches the search term
     */
    productMatchesSearch(product, searchTerm) {
        // Search in name
        if (product.name && product.name.toLowerCase().includes(searchTerm)) return true;
        
        // Search in brand
        if (product.brand && product.brand.toLowerCase().includes(searchTerm)) return true;
        
        // Search in description
        if (product.description && product.description.toLowerCase().includes(searchTerm)) return true;
        
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
        
        if (rowData && rowData[0]) { // Check if product has an ID
            const productId = rowData[0];
            const productName = rowData[1];
            
            // Open the product form in edit mode
            if (window.productForm) {
                window.productForm.openModalForEdit(productId);
            }
        }
    }

    /**
     * Handle cell changes - update product data
     */
    async onCellChange(changes, source) {
        if (source === 'loadData') return;

        for (const change of changes) {
            const [row, prop, oldValue, newValue] = change;

            if (oldValue === newValue) continue;

            const rowData = this.hotInstance.getDataAtRow(row);
            const productId = rowData[0]; // ID is in first column

            if (!productId) {
                // Skip rows without ID (shouldn't happen with modal approach)
                continue;
            }

            try {
                // Prepare update data
                const updateData = {};
                updateData[prop] = newValue;

                await apiClient.updateProduct(productId, updateData);

            } catch (error) {
                console.error('Failed to update product:', error);
                this.showError(`Failed to update product: ${error.message}`);

                // Revert the change
                this.hotInstance.setDataAtCell(row, this.hotInstance.propToCol(prop), oldValue);
            }
        }
    }

    /**
     * Capture product data before row removal and show confirmation
     */
    beforeRowRemove(index, amount, physicalRows, source) {
        // Store the products that will be deleted
        this.productsToDelete = [];

        for (const physicalRow of physicalRows) {
            // Get the actual displayed row data at the time of deletion
            const rowData = this.hotInstance.getDataAtRow(physicalRow);

            if (rowData && rowData[0]) { // ID is in column 0
                const productId = rowData[0];
                const productName = rowData[1]; // Name is in column 1

                this.productsToDelete.push({
                    id: productId,
                    name: productName,
                    physicalRow: physicalRow
                });
            }
        }

        // Show confirmation dialog
        if (this.productsToDelete.length > 0) {
            const names = this.productsToDelete.map(product => `"${product.name}"`).join(', ');
            const pluralText = this.productsToDelete.length === 1 ? 'product' : 'products';
            const confirmed = confirm(
                `Are you sure you want to delete ${this.productsToDelete.length} ${pluralText}: ${names}?\n\n` +
                `This will also delete all related meals that use ${this.productsToDelete.length === 1 ? 'this product' : 'these products'}.`
            );

            if (!confirmed) {
                // Cancel the deletion by clearing the list
                this.productsToDelete = [];
                return false;
            }
        }
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.productsToDelete || this.productsToDelete.length === 0) {
            // User cancelled or no products to delete, refresh to restore rows
            await this.loadData();
            return;
        }

        // Delete each product from the backend
        for (const product of this.productsToDelete) {
            try {
                await apiClient.deleteProduct(product.id);
            } catch (error) {
                console.error('Failed to delete product:', error);
                this.showError(`Failed to delete product "${product.name}": ${error.message}`);
                // Refresh data to restore the state
                await this.loadData();
                return;
            }
        }

        // Clear the temporary storage
        this.productsToDelete = [];

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

// Make ProductSpreadsheet globally available
window.ProductSpreadsheet = ProductSpreadsheet;