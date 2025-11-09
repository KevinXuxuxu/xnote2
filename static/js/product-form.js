/**
 * Product Form Component for managing product creation modal
 */
class ProductForm {
    constructor() {
        this.modal = document.getElementById('productModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.saveAndAddBtn = document.getElementById('saveAndAddBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');

        this.currentProductId = null;
        this.setupEventListeners();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveProduct(false);
        this.saveAndAddBtn.onclick = () => this.saveProduct(true);

        // Close modal when clicking outside
        window.onclick = (event) => {
            if (event.target === this.modal) {
                this.closeModal();
            }
        };

        // Close modal when pressing ESC key
        document.addEventListener('keydown', (event) => {
            if (event.key === 'Escape' && this.modal.style.display === 'block') {
                this.closeModal();
            }
        });
    }

    openModal() {
        this.currentProductId = null;
        this.modalTitle.textContent = 'Add Product';
        this.saveAndAddBtn.textContent = 'Save and add another';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    async openModalForEdit(productId) {
        try {
            this.currentProductId = productId;
            this.modalTitle.textContent = 'Edit Product';
            this.saveAndAddBtn.textContent = 'Save as new';
            
            // Load existing product data
            const product = await apiClient.getProduct(productId);
            this.renderForm(product);
            this.modal.style.display = 'block';
        } catch (error) {
            console.error('Failed to load product for editing:', error);
            alert(`Failed to load product for editing: ${error.message}`);
        }
    }

    renderForm(productData = null) {
        const name = productData?.name || '';

        this.modalBody.innerHTML = `
            <form id="productForm">
                <div class="form-group">
                    <label for="productName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="productName" name="name" required placeholder="Enter product name" value="${name}">
                </div>
            </form>
        `;

        // Focus on the name field
        setTimeout(() => {
            document.getElementById('productName').focus();
        }, 100);
    }

    collectFormData() {
        const form = document.getElementById('productForm');
        const formData = new FormData(form);
        const data = {};

        for (const [key, value] of formData.entries()) {
            data[key] = value.trim();
        }

        return data;
    }

    validateForm(data) {
        const errors = [];

        if (!data.name) {
            errors.push('Product name is required');
        }

        return errors;
    }

    async saveProduct(saveAsNew = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            const isEditing = this.currentProductId !== null;
            const shouldCreateNew = !isEditing || saveAsNew;

            if (shouldCreateNew) {
                // Check for potential duplicates only when creating new products
                const existingProducts = await apiClient.getProducts();
                const duplicates = window.duplicateUtils.findPotentialDuplicates(formData.name, existingProducts);
                
                if (duplicates.length > 0) {
                    const confirmed = window.duplicateUtils.showDuplicateConfirmation('Product', formData.name, duplicates);
                    if (!confirmed) {
                        return; // User cancelled the creation
                    }
                }

                await apiClient.createProduct(formData);
            } else {
                // Update existing product
                await apiClient.updateProduct(this.currentProductId, formData);
            }

            // Refresh the product spreadsheet
            if (window.productSpreadsheet) {
                window.productSpreadsheet.refresh();
            }

            if (saveAsNew) {
                // Save as new: keep modal open and reset form with current data
                this.currentProductId = null;
                this.modalTitle.textContent = 'Add Product';
                this.saveAndAddBtn.textContent = 'Save and add another';
                this.renderForm(formData); // Pre-fill with current data
            } else if (isEditing) {
                // Edit mode: close modal after update
                this.closeModal();
            } else {
                // New mode: close modal after creation
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to save product:', error);
            alert(`Failed to save product: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
        this.currentProductId = null;
    }
}

// Make ProductForm globally available
window.ProductForm = ProductForm;