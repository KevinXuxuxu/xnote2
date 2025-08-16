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
        this.peopleChoices = null; // Store Choices.js instance for people
        
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
                        <label>Meal Time</label>
                        <div class="choice-list">
                            <label class="choice-option">
                                <input type="radio" name="time" value="breakfast" ${data.time === 'breakfast' ? 'checked' : ''} required>
                                <span class="choice-label">Breakfast</span>
                            </label>
                            <label class="choice-option">
                                <input type="radio" name="time" value="lunch" ${data.time === 'lunch' ? 'checked' : ''} required>
                                <span class="choice-label">Lunch</span>
                            </label>
                            <label class="choice-option">
                                <input type="radio" name="time" value="dinner" ${data.time === 'dinner' ? 'checked' : ''} required>
                                <span class="choice-label">Dinner</span>
                            </label>
                        </div>
                    </div>
                </div>
                
                <div class="form-group">
                    <label>Food Source</label>
                    <div class="choice-list">
                        <label class="choice-option">
                            <input type="radio" name="foodSourceType" value="recipe" ${data.food_source?.type === 'recipe' ? 'checked' : ''} required>
                            <span class="choice-label">Recipe</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="foodSourceType" value="product" ${data.food_source?.type === 'product' ? 'checked' : ''} required>
                            <span class="choice-label">Product</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="foodSourceType" value="restaurant" ${data.food_source?.type === 'restaurant' ? 'checked' : ''} required>
                            <span class="choice-label">Restaurant</span>
                        </label>
                    </div>
                </div>
                
                <div id="foodSourceDetails"></div>
                
                <div class="form-group">
                    <label>Meal Type</label>
                    <div class="choice-list">
                        <label class="choice-option">
                            <input type="radio" name="mealType" value="cooked" ${this.getFoodSourceMealType() === 'cooked' ? 'checked' : ''}>
                            <span class="choice-label">Cooked</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="mealType" value="dine-in" ${this.getFoodSourceMealType() === 'dine-in' ? 'checked' : ''}>
                            <span class="choice-label">Dine-in</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="mealType" value="takeout" ${this.getFoodSourceMealType() === 'takeout' ? 'checked' : ''}>
                            <span class="choice-label">Takeout</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="mealType" value="manufactured" ${this.getFoodSourceMealType() === 'manufactured' ? 'checked' : ''}>
                            <span class="choice-label">Manufactured</span>
                        </label>
                        <label class="choice-option">
                            <input type="radio" name="mealType" value="leftover" ${this.getFoodSourceMealType() === 'leftover' ? 'checked' : ''}>
                            <span class="choice-label">Leftover</span>
                        </label>
                    </div>
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
        const foodSourceRadios = this.modalBody.querySelectorAll('input[name="foodSourceType"]');
        foodSourceRadios.forEach(radio => {
            radio.onchange = (e) => {
                if (e.target.checked) {
                    this.renderFoodSourceDetails(e.target.value);
                }
            };
        });
        
        // Render initial food source details
        if (data.food_source) {
            this.renderFoodSourceDetails(data.food_source.type);
        }
        
        // Initialize Choices.js for all selectors
        this.initializeChoicesSelectors();
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
        
        // Initialize Choices.js for people selector
        this.initializeChoicesSelectors();
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
        
        // Initialize Choices.js for people selector
        this.initializeChoicesSelectors();
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
            alert(`Failed to save changes: ${error.message}`);
        }
    }

    collectFormData() {
        const form = this.modalBody.querySelector('form');
        const formData = new FormData(form);
        const data = {};
        
        for (const [key, value] of formData.entries()) {
            data[key] = value;
        }
        
        // Handle multiple select for people (using Choices.js)
        if (this.peopleChoices) {
            data.people = this.peopleChoices.getValue(true).map(value => parseInt(value));
        } else {
            // Fallback for regular select
            const peopleSelect = form.querySelector('[name="people"]');
            if (peopleSelect) {
                data.people = Array.from(peopleSelect.selectedOptions).map(option => parseInt(option.value));
            }
        }
        
        // Transform data based on event type
        if (this.currentEventType === 'meal') {
            return this.transformMealData(data);
        } else if (this.currentEventType === 'event') {
            return this.transformEventData(data);
        } else if (this.currentEventType === 'drink') {
            return this.transformDrinkData(data);
        }
        
        return data;
    }

    transformMealData(data) {
        // console.log('Raw form data:', data);
        
        const transformed = {
            date: data.date,
            time: data.time,
            notes: data.notes || null,
            people_ids: data.people || []
        };

        // Transform food source based on type
        const foodSourceType = data.foodSourceType;
        const mealType = data.mealType || 'cooked';

        // console.log('Food source type:', foodSourceType);
        // console.log('Recipe value:', data.recipe);
        // console.log('Product value:', data.product);
        // console.log('Restaurant value:', data.restaurant);

        if (foodSourceType === 'recipe' && data.recipe) {
            transformed.food_source = {
                type: 'recipe',
                recipe_id: parseInt(data.recipe),
                meal_type: mealType
            };
        } else if (foodSourceType === 'product' && data.product) {
            transformed.food_source = {
                type: 'product',
                product_id: parseInt(data.product),
                meal_type: mealType
            };
        } else if (foodSourceType === 'restaurant' && data.restaurant) {
            transformed.food_source = {
                type: 'restaurant',
                restaurant_id: parseInt(data.restaurant),
                meal_type: mealType
            };
        } else {
            console.error('Food source validation failed:', {
                foodSourceType,
                recipe: data.recipe,
                product: data.product,
                restaurant: data.restaurant
            });
            throw new Error('Please select a food source (recipe, product, or restaurant) and choose a specific item');
        }

        // console.log('Transformed meal data:', transformed);
        return transformed;
    }

    transformEventData(data) {
        return {
            date: data.date,
            activity_id: parseInt(data.activity),
            measure: data.measure || null,
            location: data.location || null,
            notes: data.notes || null,
            people_ids: data.people || []
        };
    }

    transformDrinkData(data) {
        return {
            date: data.date,
            name: data.name,
            people_ids: data.people || []
        };
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

    initializeChoicesSelectors() {
        // Destroy existing Choices instance if it exists
        if (this.peopleChoices) {
            this.peopleChoices.destroy();
            this.peopleChoices = null;
        }
        
        // Initialize people selector (multi-select with search)
        const peopleSelect = this.modalBody.querySelector('select[name="people"]');
        if (peopleSelect) {
            this.peopleChoices = new Choices(peopleSelect, {
                removeItemButton: true,
                searchEnabled: true,
                searchPlaceholderValue: 'Search people...',
                placeholderValue: 'Choose people',
                noResultsText: 'No people found',
                itemSelectText: '',
            });
        }
    }

    closeModal() {
        // Destroy Choices instance when closing modal
        if (this.peopleChoices) {
            this.peopleChoices.destroy();
            this.peopleChoices = null;
        }
        
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