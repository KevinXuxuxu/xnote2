/**
 * Activity Form Component for managing activity creation modal
 */
class ActivityForm {
    constructor() {
        this.modal = document.getElementById('activityModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.saveAndAddBtn = document.getElementById('saveAndAddBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');
        
        this.activityTypeChoices = null;
        
        this.setupEventListeners();
        this.loadEnumData();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveActivity(false);
        this.saveAndAddBtn.onclick = () => this.saveActivity(true);
        
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

    async loadEnumData() {
        try {
            // Load activity types for dropdown
            const activityTypes = await apiClient.getActivityTypes().catch(err => {
                console.warn('Failed to load activity types:', err);
                return [];
            });
            
            this.enumData = {
                activityTypes: activityTypes || []
            };
        } catch (error) {
            console.error('Failed to load enum data:', error);
            this.enumData = {
                activityTypes: []
            };
        }
    }

    openModal() {
        this.modalTitle.textContent = 'Add Activity';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="activityForm">
                <div class="form-group">
                    <label for="activityName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="activityName" name="name" required placeholder="Enter activity name">
                </div>
                
                <div class="form-group">
                    <label for="activityType">Activity Type <span style="color: red;">*</span></label>
                    <select id="activityType" name="type" required>
                        <option value="">Select or enter activity type</option>
                        ${this.enumData.activityTypes.map(activityType => 
                            `<option value="${activityType.name}">${activityType.name}</option>`
                        ).join('')}
                    </select>
                </div>
            </form>
        `;
        
        // Initialize Choices.js for activity type selector
        this.initializeChoices();
        
        // Focus on the name field
        setTimeout(() => {
            document.getElementById('activityName').focus();
        }, 100);
    }

    initializeChoices() {
        // Destroy existing instance
        if (this.activityTypeChoices) {
            this.activityTypeChoices.destroy();
            this.activityTypeChoices = null;
        }
        
        // Initialize activity type selector (required, with custom values)
        const activityTypeSelect = document.getElementById('activityType');
        if (activityTypeSelect) {
            this.activityTypeChoices = new Choices(activityTypeSelect, {
                searchEnabled: true,
                searchPlaceholderValue: 'Search or type activity type...',
                placeholderValue: 'Choose or enter activity type',
                noResultsText: 'No activity types found',
                itemSelectText: '',
                addItems: true,
                editItems: true,
                allowHTML: false,
            });
        }
    }

    collectFormData() {
        const form = document.getElementById('activityForm');
        const formData = new FormData(form);
        const data = {};
        
        for (const [key, value] of formData.entries()) {
            data[key] = value.trim();
        }
        
        // Override with Choices.js value if available
        if (this.activityTypeChoices) {
            const activityTypeValue = this.activityTypeChoices.getValue(true);
            data.type = activityTypeValue || '';
        }
        
        return data;
    }

    validateForm(data) {
        const errors = [];
        
        if (!data.name) {
            errors.push('Activity name is required');
        }
        if (!data.type) {
            errors.push('Activity type is required');
        }
        
        return errors;
    }

    async saveActivity(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);
            
            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }
            
            await apiClient.createActivity(formData);
            
            // Refresh the activity spreadsheet
            if (window.activitySpreadsheet) {
                window.activitySpreadsheet.refresh();
            }
            
            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }
            
        } catch (error) {
            console.error('Failed to create activity:', error);
            alert(`Failed to create activity: ${error.message}`);
        }
    }

    closeModal() {
        // Destroy Choices instance when closing modal
        if (this.activityTypeChoices) {
            this.activityTypeChoices.destroy();
            this.activityTypeChoices = null;
        }
        
        this.modal.style.display = 'none';
    }
}

// Make ActivityForm globally available
window.ActivityForm = ActivityForm;