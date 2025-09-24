/**
 * Restaurant Form Component for managing restaurant creation modal
 */
class RestaurantForm {
    constructor() {
        this.modal = document.getElementById('restaurantModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.saveAndAddBtn = document.getElementById('saveAndAddBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');

        this.locationChoices = null;
        this.foodTypeChoices = null;

        this.setupEventListeners();
        this.loadEnumData();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveRestaurant(false);
        this.saveAndAddBtn.onclick = () => this.saveRestaurant(true);

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
            // Load locations and food types for dropdowns
            const [locations, foodTypes] = await Promise.all([
                apiClient.getLocations().catch(err => {
                    console.warn('Failed to load locations:', err);
                    return [];
                }),
                apiClient.getFoodTypes().catch(err => {
                    console.warn('Failed to load food types:', err);
                    return [];
                })
            ]);

            this.enumData = {
                locations: locations || [],
                foodTypes: foodTypes || []
            };
        } catch (error) {
            console.error('Failed to load enum data:', error);
            this.enumData = {
                locations: [],
                foodTypes: []
            };
        }
    }

    openModal() {
        this.modalTitle.textContent = 'Add Restaurant';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="restaurantForm">
                <div class="form-group">
                    <label for="restaurantName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="restaurantName" name="name" required placeholder="Enter restaurant name">
                </div>
                
                <div class="form-group">
                    <label for="restaurantLocation">Location</label>
                    <select id="restaurantLocation" name="location">
                        <option value="">Select or enter location</option>
                        ${this.enumData.locations.map(location =>
            `<option value="${location.name}">${location.name}</option>`
        ).join('')}
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="restaurantType">Food Type <span style="color: red;">*</span></label>
                    <select id="restaurantType" name="type" required>
                        <option value="">Select or enter food type</option>
                        ${this.enumData.foodTypes.map(foodType =>
            `<option value="${foodType.name}">${foodType.name}</option>`
        ).join('')}
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="restaurantPrice">Price (Optional)</label>
                    <input type="number" id="restaurantPrice" name="price" step="0.01" min="0" placeholder="Average price">
                </div>
            </form>
        `;

        // Initialize Choices.js for selectors
        this.initializeChoices();

        // Focus on the name field
        setTimeout(() => {
            document.getElementById('restaurantName').focus();
        }, 100);
    }

    initializeChoices() {
        // Destroy existing instances
        if (this.locationChoices) {
            this.locationChoices.destroy();
            this.locationChoices = null;
        }
        if (this.foodTypeChoices) {
            this.foodTypeChoices.destroy();
            this.foodTypeChoices = null;
        }

        // Initialize location selector (optional, with custom values)
        const locationSelect = document.getElementById('restaurantLocation');
        if (locationSelect) {
            this.locationChoices = new Choices(locationSelect, {
                searchEnabled: true,
                searchPlaceholderValue: 'Search or type location...',
                placeholderValue: 'Choose or enter location',
                noResultsText: 'No locations found',
                itemSelectText: '',
                addItems: true,
                editItems: true,
                allowHTML: false,
            });
        }

        // Initialize food type selector (required, with custom values)
        const foodTypeSelect = document.getElementById('restaurantType');
        if (foodTypeSelect) {
            this.foodTypeChoices = new Choices(foodTypeSelect, {
                searchEnabled: true,
                searchPlaceholderValue: 'Search or type food type...',
                placeholderValue: 'Choose or enter food type',
                noResultsText: 'No food types found',
                itemSelectText: '',
                addItems: true,
                editItems: true,
                allowHTML: false,
            });
        }
    }

    collectFormData() {
        const form = document.getElementById('restaurantForm');
        const formData = new FormData(form);
        const data = {};

        for (const [key, value] of formData.entries()) {
            if (key === 'price') {
                data[key] = value ? parseFloat(value) : null;
            } else {
                data[key] = value.trim();
            }
        }

        // Override with Choices.js values if available
        if (this.locationChoices) {
            const locationValue = this.locationChoices.getValue(true);
            data.location = locationValue || '';
        }
        if (this.foodTypeChoices) {
            const foodTypeValue = this.foodTypeChoices.getValue(true);
            data.type = foodTypeValue || '';
        }

        return data;
    }

    validateForm(data) {
        const errors = [];

        if (!data.name) {
            errors.push('Restaurant name is required');
        }
        if (!data.type) {
            errors.push('Food type is required');
        }
        if (data.price !== null && data.price < 0) {
            errors.push('Price cannot be negative');
        }

        return errors;
    }

    async saveRestaurant(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            // Check for potential duplicates
            const existingRestaurants = await apiClient.getRestaurants();
            const duplicates = window.duplicateUtils.findPotentialDuplicates(formData.name, existingRestaurants);
            
            if (duplicates.length > 0) {
                const confirmed = window.duplicateUtils.showDuplicateConfirmation('Restaurant', formData.name, duplicates);
                if (!confirmed) {
                    return; // User cancelled the creation
                }
            }

            await apiClient.createRestaurant(formData);

            // Refresh the restaurant spreadsheet
            if (window.restaurantSpreadsheet) {
                window.restaurantSpreadsheet.refresh();
            }

            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to create restaurant:', error);
            alert(`Failed to create restaurant: ${error.message}`);
        }
    }

    closeModal() {
        // Destroy Choices instances when closing modal
        if (this.locationChoices) {
            this.locationChoices.destroy();
            this.locationChoices = null;
        }
        if (this.foodTypeChoices) {
            this.foodTypeChoices.destroy();
            this.foodTypeChoices = null;
        }

        this.modal.style.display = 'none';
    }
}

// Make RestaurantForm globally available
window.RestaurantForm = RestaurantForm;