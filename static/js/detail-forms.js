/**
 * Detail Forms Component for managing modal interactions
 */
class DetailForms {
    constructor() {
        this.modal = document.getElementById('detailModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');
        
        this.currentEventId = null;
        this.currentEventType = null;
        this.currentData = null;
        
        this.setupEventListeners();
        this.loadEnumData();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveChanges();
        
        // Close modal when clicking outside
        window.onclick = (event) => {
            if (event.target === this.modal) {
                this.closeModal();
            }
        };
    }

    async loadEnumData() {
        try {
            // Load enum data for dropdowns
            this.enumData = {
                people: await apiClient.getPeople(),
                locations: await apiClient.getLocations(),
                restaurants: await apiClient.getRestaurants(),
                recipes: await apiClient.getRecipes(),
                products: await apiClient.getProducts(),
                activities: await apiClient.getActivities(),
                drinkOptions: await apiClient.getDrinkOptions()
            };
        } catch (error) {
            console.error('Failed to load enum data:', error);
            this.enumData = {
                people: [],
                locations: [],
                restaurants: [],
                recipes: [],
                products: [],
                activities: [],
                drinkOptions: []
            };
        }
    }

    async openDetails(id, eventType) {
        this.currentEventId = id;
        this.currentEventType = eventType;
        
        try {
            if (id) {
                // Edit existing event
                this.currentData = await apiClient.getEventDetails(id, eventType);
                this.modalTitle.textContent = `Edit ${this.capitalize(eventType)}`;
            } else {
                // Create new event
                this.currentData = this.getDefaultData(eventType);
                this.modalTitle.textContent = `New ${this.capitalize(eventType)}`;
            }
            
            this.renderForm();
            this.modal.style.display = 'block';
        } catch (error) {
            console.error('Failed to load event details:', error);
            alert('Failed to load event details');
        }
    }

    getDefaultData(eventType) {
        const today = new Date().toISOString().split('T')[0];
        
        switch (eventType) {
            case 'meal':
                return {
                    date: today,
                    time: 'lunch',
                    notes: '',
                    food_source: null,
                    people: []
                };
            case 'event':
                return {
                    date: today,
                    activity: null,
                    measure: '',
                    location: '',
                    notes: '',
                    people: []
                };
            case 'drink':
                return {
                    date: today,
                    name: '',
                    people: []
                };
            default:
                return {};
        }
    }

    renderForm() {
        switch (this.currentEventType) {
            case 'meal':
                this.renderMealForm();
                break;
            case 'event':
                this.renderEventForm();
                break;
            case 'drink':
                this.renderDrinkForm();
                break;
        }
    }

    renderMealForm() {
        const data = this.currentData;
        
        this.modalBody.innerHTML = `
            <form id="mealForm">
                <div class="form-row">
                    <div class="form-group">
                        <label for="mealDate">Date</label>
                        <input type="date" id="mealDate" name="date" value="${data.date || ''}" required>
                    </div>
                    <div class="form-group">
                        <label for="mealTime">Meal Time</label>
                        <select id="mealTime" name="time" required>
                            <option value="breakfast" ${data.time === 'breakfast' ? 'selected' : ''}>Breakfast</option>
                            <option value="lunch" ${data.time === 'lunch' ? 'selected' : ''}>Lunch</option>
                            <option value="dinner" ${data.time === 'dinner' ? 'selected' : ''}>Dinner</option>
                        </select>
                    </div>
                </div>
                
                <div class="form-group">
                    <label for="foodSource">Food Source</label>
                    <select id="foodSourceType" name="foodSourceType">
                        <option value="">Select food source type</option>
                        <option value="recipe" ${data.food_source?.type === 'recipe' ? 'selected' : ''}>Recipe</option>
                        <option value="product" ${data.food_source?.type === 'product' ? 'selected' : ''}>Product</option>
                        <option value="restaurant" ${data.food_source?.type === 'restaurant' ? 'selected' : ''}>Restaurant</option>
                    </select>
                </div>
                
                <div id="foodSourceDetails"></div>
                
                <div class="form-group">
                    <label for="mealType">Meal Type</label>
                    <select id="mealType" name="mealType">
                        <option value="cooked" ${this.getFoodSourceMealType() === 'cooked' ? 'selected' : ''}>Cooked</option>
                        <option value="dine-in" ${this.getFoodSourceMealType() === 'dine-in' ? 'selected' : ''}>Dine-in</option>
                        <option value="takeout" ${this.getFoodSourceMealType() === 'takeout' ? 'selected' : ''}>Takeout</option>
                        <option value="manufactured" ${this.getFoodSourceMealType() === 'manufactured' ? 'selected' : ''}>Manufactured</option>
                        <option value="leftover" ${this.getFoodSourceMealType() === 'leftover' ? 'selected' : ''}>Leftover</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="mealPeople">People</label>
                    <select id="mealPeople" name="people" multiple>
                        ${this.renderPeopleOptions(data.people)}
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="mealNotes">Notes</label>
                    <textarea id="mealNotes" name="notes" placeholder="Additional notes...">${data.notes || ''}</textarea>
                </div>
            </form>
        `;
        
        // Setup food source type change handler
        document.getElementById('foodSourceType').onchange = (e) => {
            this.renderFoodSourceDetails(e.target.value);
        };
        
        // Render initial food source details
        if (data.food_source) {
            this.renderFoodSourceDetails(data.food_source.type);
        }
    }

    getFoodSourceMealType() {
        const data = this.currentData;
        if (data.food_source && data.food_source.details) {
            return data.food_source.details.meal_type;
        }
        return 'cooked';
    }

    renderFoodSourceDetails(type) {
        const container = document.getElementById('foodSourceDetails');
        const data = this.currentData;
        
        switch (type) {
            case 'recipe':
                container.innerHTML = `
                    <div class="form-group">
                        <label for="recipeSelect">Recipe</label>
                        <select id="recipeSelect" name="recipe">
                            <option value="">Select a recipe</option>
                            ${this.enumData.recipes.map(recipe => 
                                `<option value="${recipe.id}" ${data.food_source?.details?.recipe?.id === recipe.id ? 'selected' : ''}>${recipe.name}</option>`
                            ).join('')}
                        </select>
                    </div>
                `;
                break;
            case 'product':
                container.innerHTML = `
                    <div class="form-group">
                        <label for="productSelect">Product</label>
                        <select id="productSelect" name="product">
                            <option value="">Select a product</option>
                            ${this.enumData.products.map(product => 
                                `<option value="${product.id}" ${data.food_source?.details?.product?.id === product.id ? 'selected' : ''}>${product.name}</option>`
                            ).join('')}
                        </select>
                    </div>
                `;
                break;
            case 'restaurant':
                container.innerHTML = `
                    <div class="form-group">
                        <label for="restaurantSelect">Restaurant</label>
                        <select id="restaurantSelect" name="restaurant">
                            <option value="">Select a restaurant</option>
                            ${this.enumData.restaurants.map(restaurant => 
                                `<option value="${restaurant.id}" ${data.food_source?.details?.restaurant?.id === restaurant.id ? 'selected' : ''}>${restaurant.name}</option>`
                            ).join('')}
                        </select>
                    </div>
                `;
                break;
            default:
                container.innerHTML = '';
        }
    }

    renderEventForm() {
        const data = this.currentData;
        
        this.modalBody.innerHTML = `
            <form id="eventForm">
                <div class="form-row">
                    <div class="form-group">
                        <label for="eventDate">Date</label>
                        <input type="date" id="eventDate" name="date" value="${data.date || ''}" required>
                    </div>
                    <div class="form-group">
                        <label for="eventActivity">Activity</label>
                        <select id="eventActivity" name="activity" required>
                            <option value="">Select an activity</option>
                            ${this.enumData.activities.map(activity => 
                                `<option value="${activity.id}" ${data.activity?.id === activity.id ? 'selected' : ''}>${activity.name} (${activity.type})</option>`
                            ).join('')}
                        </select>
                    </div>
                </div>
                
                <div class="form-row">
                    <div class="form-group">
                        <label for="eventMeasure">Measure</label>
                        <input type="text" id="eventMeasure" name="measure" value="${data.measure || ''}" placeholder="e.g., 30 minutes, 5 km">
                    </div>
                    <div class="form-group">
                        <label for="eventLocation">Location</label>
                        <input type="text" id="eventLocation" name="location" value="${data.location || ''}" list="locationsList">
                        <datalist id="locationsList">
                            ${this.enumData.locations.map(location => 
                                `<option value="${location.name}">`
                            ).join('')}
                        </datalist>
                    </div>
                </div>
                
                <div class="form-group">
                    <label for="eventPeople">People</label>
                    <select id="eventPeople" name="people" multiple>
                        ${this.renderPeopleOptions(data.people)}
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="eventNotes">Notes</label>
                    <textarea id="eventNotes" name="notes" placeholder="Additional notes...">${data.notes || ''}</textarea>
                </div>
            </form>
        `;
    }

    renderDrinkForm() {
        const data = this.currentData;
        
        this.modalBody.innerHTML = `
            <form id="drinkForm">
                <div class="form-row">
                    <div class="form-group">
                        <label for="drinkDate">Date</label>
                        <input type="date" id="drinkDate" name="date" value="${data.date || ''}" required>
                    </div>
                    <div class="form-group">
                        <label for="drinkName">Drink</label>
                        <select id="drinkName" name="name" required>
                            <option value="">Select a drink</option>
                            ${this.enumData.drinkOptions.map(option => 
                                `<option value="${option.name}" ${data.name === option.name ? 'selected' : ''}>${option.name}</option>`
                            ).join('')}
                        </select>
                    </div>
                </div>
                
                <div class="form-group">
                    <label for="drinkPeople">People</label>
                    <select id="drinkPeople" name="people" multiple>
                        ${this.renderPeopleOptions(data.people)}
                    </select>
                </div>
            </form>
        `;
    }

    renderPeopleOptions(selectedPeople = []) {
        const selectedIds = selectedPeople.map(p => p.id);
        return this.enumData.people.map(person => 
            `<option value="${person.id}" ${selectedIds.includes(person.id) ? 'selected' : ''}>${person.name}</option>`
        ).join('');
    }

    async saveChanges() {
        try {
            const formData = this.collectFormData();
            
            if (this.currentEventId) {
                // Update existing event
                await this.updateEvent(formData);
            } else {
                // Create new event
                await this.createEvent(formData);
            }
            
            this.closeModal();
            
            // Refresh the spreadsheet
            if (window.eventSpreadsheet) {
                window.eventSpreadsheet.refresh();
            }
            
        } catch (error) {
            console.error('Failed to save changes:', error);
            alert('Failed to save changes');
        }
    }

    collectFormData() {
        const form = this.modalBody.querySelector('form');
        const formData = new FormData(form);
        const data = {};
        
        for (const [key, value] of formData.entries()) {
            data[key] = value;
        }
        
        // Handle multiple select for people
        const peopleSelect = form.querySelector('[name="people"]');
        if (peopleSelect) {
            data.people = Array.from(peopleSelect.selectedOptions).map(option => parseInt(option.value));
        }
        
        return data;
    }

    async createEvent(formData) {
        switch (this.currentEventType) {
            case 'meal':
                return await apiClient.createMeal(formData);
            case 'event':
                return await apiClient.createEvent(formData);
            case 'drink':
                return await apiClient.createDrink(formData);
        }
    }

    async updateEvent(formData) {
        switch (this.currentEventType) {
            case 'meal':
                return await apiClient.updateMeal(this.currentEventId, formData);
            case 'event':
                return await apiClient.updateEvent(this.currentEventId, formData);
            case 'drink':
                return await apiClient.updateDrink(this.currentEventId, formData);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
        this.currentEventId = null;
        this.currentEventType = null;
        this.currentData = null;
    }

    capitalize(str) {
        return str.charAt(0).toUpperCase() + str.slice(1);
    }
}

// Initialize DetailForms globally
window.detailForms = new DetailForms();