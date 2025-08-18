/**
 * Product Spreadsheet Component using Handsontable
 */
class ProductSpreadsheet {
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
            
            const products = await apiClient.getProducts();
            this.data = products;
            this.hotInstance.loadData(this.data);
            
            this.showLoading(false);
        } catch (error) {
            console.error('Failed to load products:', error);
            this.showError('Failed to load products from server');
            this.showLoading(false);
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
     * Capture product data before row removal
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
    }

    /**
     * Handle row removal aftermath - delete from backend
     */
    async afterRowRemove(index, amount, physicalRows, source) {
        if (!this.productsToDelete || this.productsToDelete.length === 0) {
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