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
        this.modalTitle.textContent = 'Add Product';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="productForm">
                <div class="form-group">
                    <label for="productName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="productName" name="name" required placeholder="Enter product name">
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

    async saveProduct(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            await apiClient.createProduct(formData);

            // Refresh the product spreadsheet
            if (window.productSpreadsheet) {
                window.productSpreadsheet.refresh();
            }

            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to create product:', error);
            alert(`Failed to create product: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
    }
}

// Make ProductForm globally available
window.ProductForm = ProductForm;