/**
 * Drink Option Form Component for managing drink option creation modal
 */
class DrinkOptionForm {
    constructor() {
        this.modal = document.getElementById('drinkOptionModal');
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
        this.saveBtn.onclick = () => this.saveDrinkOption(false);
        this.saveAndAddBtn.onclick = () => this.saveDrinkOption(true);

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
        this.modalTitle.textContent = 'Add Drink Option';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="drinkOptionForm">
                <div class="form-group">
                    <label for="drinkOptionName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="drinkOptionName" name="name" required placeholder="Enter drink option name">
                </div>
            </form>
        `;

        // Focus on the name field
        setTimeout(() => {
            document.getElementById('drinkOptionName').focus();
        }, 100);
    }

    collectFormData() {
        const form = document.getElementById('drinkOptionForm');
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
            errors.push('Drink option name is required');
        }

        return errors;
    }

    async saveDrinkOption(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            await apiClient.createDrinkOption(formData);

            // Refresh the drink option spreadsheet
            if (window.drinkOptionSpreadsheet) {
                window.drinkOptionSpreadsheet.refresh();
            }

            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to create drink option:', error);
            alert(`Failed to create drink option: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
    }
}

// Make DrinkOptionForm globally available
window.DrinkOptionForm = DrinkOptionForm;