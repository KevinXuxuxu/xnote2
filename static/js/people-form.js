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

        this.currentPersonId = null;

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
        this.currentPersonId = null;
        this.modalTitle.textContent = 'Add Person';
        this.saveAndAddBtn.textContent = 'Save and add another';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    async openModalForEdit(personId) {
        try {
            this.currentPersonId = personId;
            this.modalTitle.textContent = 'Edit Person';
            this.saveAndAddBtn.textContent = 'Save as new';
            
            // Load existing person data
            const person = await apiClient.getPerson(personId);
            this.renderForm(person);
            this.modal.style.display = 'block';
        } catch (error) {
            console.error('Failed to load person for editing:', error);
            alert(`Failed to load person for editing: ${error.message}`);
        }
    }

    renderForm(personData = null) {
        const name = personData?.name || '';
        const notes = personData?.notes || '';

        this.modalBody.innerHTML = `
            <form id="personForm">
                <div class="form-group">
                    <label for="personName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="personName" name="name" required placeholder="Enter person's name" value="${name}">
                </div>
                
                <div class="form-group">
                    <label for="personNotes">Notes (Optional)</label>
                    <textarea id="personNotes" name="notes" rows="4" 
                              placeholder="Any additional information about this person">${notes}</textarea>
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

    async savePerson(saveAsNew = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            const isEditing = this.currentPersonId !== null;
            const shouldCreateNew = !isEditing || saveAsNew;

            if (shouldCreateNew) {
                // Check for potential duplicates only when creating new people
                const existingPeople = await apiClient.getPeople();
                const duplicates = window.duplicateUtils.findPotentialDuplicates(formData.name, existingPeople);
                
                if (duplicates.length > 0) {
                    const confirmed = window.duplicateUtils.showDuplicateConfirmation('Person', formData.name, duplicates);
                    if (!confirmed) {
                        return; // User cancelled the creation
                    }
                }

                await apiClient.createPerson(formData);
            } else {
                // Update existing person
                await apiClient.updatePerson(this.currentPersonId, formData);
            }

            // Refresh the people spreadsheet
            if (window.peopleSpreadsheet) {
                window.peopleSpreadsheet.refresh();
            }

            if (saveAsNew) {
                // Save as new: keep modal open and reset form with current data
                this.currentPersonId = null;
                this.modalTitle.textContent = 'Add Person';
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
            console.error('Failed to save person:', error);
            alert(`Failed to save person: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
        this.currentPersonId = null;
    }
}

// Make PeopleForm globally available
window.PeopleForm = PeopleForm;