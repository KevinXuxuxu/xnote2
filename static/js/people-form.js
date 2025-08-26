/**
 * People Form Component for managing people creation modal
 */
class PeopleForm {
    constructor() {
        this.modal = document.getElementById('personModal');
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
        this.saveBtn.onclick = () => this.savePerson(false);
        this.saveAndAddBtn.onclick = () => this.savePerson(true);

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
        this.modalTitle.textContent = 'Add Person';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="personForm">
                <div class="form-group">
                    <label for="personName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="personName" name="name" required placeholder="Enter person's name">
                </div>
                
                <div class="form-group">
                    <label for="personNotes">Notes (Optional)</label>
                    <textarea id="personNotes" name="notes" rows="4" 
                              placeholder="Any additional information about this person"></textarea>
                </div>
            </form>
        `;

        // Focus on the name field
        setTimeout(() => {
            document.getElementById('personName').focus();
        }, 100);
    }

    collectFormData() {
        const form = document.getElementById('personForm');
        const formData = new FormData(form);
        const data = {};

        for (const [key, value] of formData.entries()) {
            data[key] = value.trim();
        }

        // Convert empty notes to null
        if (!data.notes) {
            data.notes = null;
        }

        return data;
    }

    validateForm(data) {
        const errors = [];

        if (!data.name) {
            errors.push('Person name is required');
        }

        return errors;
    }

    async savePerson(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            await apiClient.createPerson(formData);

            // Refresh the people spreadsheet
            if (window.peopleSpreadsheet) {
                window.peopleSpreadsheet.refresh();
            }

            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to create person:', error);
            alert(`Failed to create person: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
    }
}

// Make PeopleForm globally available
window.PeopleForm = PeopleForm;