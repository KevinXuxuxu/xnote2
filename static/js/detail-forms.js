/**
 * Detail Forms Component for managing modal interactions
 */
class DetailForms {
    constructor() {
        this.modal = document.getElementById('detailModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.saveAndAddBtn = document.getElementById('saveAndAddBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');
        
        this.currentEventId = null;
        this.currentEventType = null;
        this.currentData = null;
        this.peopleChoices = null; // Store Choices.js instance for people
        this.foodSourceChoices = null; // Store Choices.js instance for food sources
        this.activityChoices = null; // Store Choices.js instance for activities
        this.drinkChoices = null; // Store Choices.js instance for drinks
        
        this.setupEventListeners();
        this.loadEnumData();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveChanges(false);
        this.saveAndAddBtn.onclick = () => this.saveChanges(true);
        
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
        const today = window.dateUtils.getTodayLocal();
        
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
                        <label for="recipeSelect">Recipes</label>
                        <select id="recipeSelect" name="recipes" multiple>
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
                        <label for="productSelect">Products</label>
                        <select id="productSelect" name="products" multiple>
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
                        <label for="restaurantSelect">Restaurants</label>
                        <select id="restaurantSelect" name="restaurants" multiple>
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
        
        // Initialize Choices.js for the food source detail selector
        this.initializeFoodSourceChoices();
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
                        <label for="eventMeasure">Measure</label>
                        <input type="text" id="eventMeasure" name="measure" value="${data.measure || ''}" placeholder="e.g., 30 minutes, 5 km">
                    </div>
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
                
                <div class="form-group">
                    <label for="eventLocation">Location</label>
                    <input type="text" id="eventLocation" name="location" value="${data.location || ''}" placeholder="Enter location">
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
        
        // Initialize Choices.js for all selectors
        this.initializeEventChoicesSelectors();
    }

    renderDrinkForm() {
        const data = this.currentData;
        
        this.modalBody.innerHTML = `
            <form id="drinkForm">
                <div class="form-group">
                    <label for="drinkDate">Date</label>
                    <input type="date" id="drinkDate" name="date" value="${data.date || ''}" required>
                </div>
                
                <div class="form-group">
                    <label for="drinkName">Drinks</label>
                    <select id="drinkName" name="drinks" multiple required>
                        ${this.enumData.drinkOptions.map(option => 
                            `<option value="${option.name}" ${data.name === option.name ? 'selected' : ''}>${option.name}</option>`
                        ).join('')}
                    </select>
                </div>
                
                <div class="form-group">
                    <label for="drinkPeople">People</label>
                    <select id="drinkPeople" name="people" multiple>
                        ${this.renderPeopleOptions(data.people)}
                    </select>
                </div>
            </form>
        `;
        
        // Initialize Choices.js for drink and people selectors
        this.initializeDrinkChoicesSelectors();
    }

    renderPeopleOptions(selectedPeople = []) {
        const selectedIds = selectedPeople.map(p => p.id);
        return this.enumData.people.map(person => 
            `<option value="${person.id}" ${selectedIds.includes(person.id) ? 'selected' : ''}>${person.name}</option>`
        ).join('');
    }

    async saveChanges(addAnother = false) {
        try {
            const formData = this.collectFormData();
            
            if (this.currentEventId) {
                // Update existing event
                await this.updateEvent(formData);
            } else {
                // Create new event
                await this.createEvent(formData);
            }
            
            // Refresh the spreadsheet
            if (window.eventSpreadsheet) {
                window.eventSpreadsheet.refresh();
            }
            
            if (addAnother) {
                // Keep modal open and reset to new form of same type
                this.openDetails(null, this.currentEventType);
            } else {
                // Close modal as usual
                this.closeModal();
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
        
        // Handle activity selection (using Choices.js)
        if (this.activityChoices) {
            const activityValue = this.activityChoices.getValue(true);
            if (activityValue) {
                data.activity = parseInt(activityValue);
            }
        }
        
        
        // Handle multiple select for drinks (using Choices.js)
        if (this.drinkChoices) {
            data.drinks = this.drinkChoices.getValue(true);
        }
        
        // Handle multiple select for food sources (using Choices.js)
        if (this.foodSourceChoices) {
            const fieldName = this.foodSourceChoices.passedElement.element.name;
            data[fieldName] = this.foodSourceChoices.getValue(true).map(value => parseInt(value));
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

    /**
     * Get people IDs with default fallback to "xx" and "ww" if empty
     */
    getPeopleIds(peopleArray) {
        if (!peopleArray || peopleArray.length === 0) {
            // Find IDs for "xx" and "ww" people
            const xxPerson = this.enumData.people.find(p => p.name.toLowerCase() === 'xx');
            const wwPerson = this.enumData.people.find(p => p.name.toLowerCase() === 'ww');
            
            const defaultIds = [];
            if (xxPerson) defaultIds.push(xxPerson.id);
            if (wwPerson) defaultIds.push(wwPerson.id);
            
            return defaultIds.length > 0 ? defaultIds : [];
        }
        return peopleArray;
    }

    transformMealData(data) {
        // Base meal data shared by all meals
        const baseMeal = {
            date: data.date,
            time: data.time,
            notes: data.notes || null,
            people_ids: this.getPeopleIds(data.people)
        };

        // Get food source information
        const foodSourceType = data.foodSourceType;
        const mealType = data.mealType || 'cooked';
        
        // Get the selected food items (recipes, products, or restaurants)
        let foodItems = [];
        if (foodSourceType === 'recipe' && data.recipes && data.recipes.length > 0) {
            foodItems = data.recipes;
        } else if (foodSourceType === 'product' && data.products && data.products.length > 0) {
            foodItems = data.products;
        } else if (foodSourceType === 'restaurant' && data.restaurants && data.restaurants.length > 0) {
            foodItems = data.restaurants;
        } else {
            throw new Error(`Please select at least one ${foodSourceType || 'food item'}`);
        }

        // Create multiple meal objects, one for each selected food item
        const meals = foodItems.map(foodId => {
            const meal = { ...baseMeal };
            
            if (foodSourceType === 'recipe') {
                meal.food_source = {
                    type: 'recipe',
                    recipe_id: parseInt(foodId),
                    meal_type: mealType
                };
            } else if (foodSourceType === 'product') {
                meal.food_source = {
                    type: 'product',
                    product_id: parseInt(foodId),
                    meal_type: mealType
                };
            } else if (foodSourceType === 'restaurant') {
                meal.food_source = {
                    type: 'restaurant',
                    restaurant_id: parseInt(foodId),
                    meal_type: mealType
                };
            }
            
            return meal;
        });

        return meals;
    }

    transformEventData(data) {
        return {
            date: data.date,
            activity_id: parseInt(data.activity),
            measure: data.measure || null,
            location: data.location || null,
            notes: data.notes || null,
            people_ids: this.getPeopleIds(data.people)
        };
    }

    transformDrinkData(data) {
        // Base drink data shared by all drinks
        const baseDrink = {
            date: data.date,
            people_ids: this.getPeopleIds(data.people)
        };

        // Get the selected drinks
        const drinkNames = data.drinks || [];
        if (drinkNames.length === 0) {
            throw new Error('Please select at least one drink');
        }

        // Create multiple drink objects, one for each selected drink
        const drinks = drinkNames.map(drinkName => ({
            ...baseDrink,
            name: drinkName
        }));

        return drinks;
    }

    async createEvent(formData) {
        switch (this.currentEventType) {
            case 'meal':
                // Handle multiple meals
                if (Array.isArray(formData)) {
                    // Create multiple meals sequentially
                    const results = [];
                    for (const meal of formData) {
                        const result = await apiClient.createMeal(meal);
                        results.push(result);
                    }
                    return results;
                } else {
                    // Single meal (fallback)
                    return await apiClient.createMeal(formData);
                }
            case 'event':
                return await apiClient.createEvent(formData);
            case 'drink':
                // Handle multiple drinks
                if (Array.isArray(formData)) {
                    // Create multiple drinks sequentially
                    const results = [];
                    for (const drink of formData) {
                        const result = await apiClient.createDrink(drink);
                        results.push(result);
                    }
                    return results;
                } else {
                    // Single drink (fallback)
                    return await apiClient.createDrink(formData);
                }
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
        
        // Initialize food source choices if they exist
        this.initializeFoodSourceChoices();
    }

    initializeFoodSourceChoices() {
        // Destroy existing food source choice instance
        if (this.foodSourceChoices) {
            this.foodSourceChoices.destroy();
            this.foodSourceChoices = null;
        }
        
        // Find any food source select element (recipes, products, or restaurants)
        const foodSourceSelect = this.modalBody.querySelector('select[name="recipes"], select[name="products"], select[name="restaurants"]');
        if (foodSourceSelect) {
            this.foodSourceChoices = new Choices(foodSourceSelect, {
                removeItemButton: true,
                searchEnabled: true,
                searchPlaceholderValue: 'Search...',
                placeholderValue: 'Choose items',
                noResultsText: 'No items found',
                itemSelectText: '',
            });
        }
    }

    initializeEventChoicesSelectors() {
        // Destroy existing Choices instances if they exist
        if (this.peopleChoices) {
            this.peopleChoices.destroy();
            this.peopleChoices = null;
        }
        if (this.activityChoices) {
            this.activityChoices.destroy();
            this.activityChoices = null;
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
        
        // Initialize activity selector (single-select with search)
        const activitySelect = this.modalBody.querySelector('select[name="activity"]');
        if (activitySelect) {
            this.activityChoices = new Choices(activitySelect, {
                searchEnabled: true,
                searchPlaceholderValue: 'Search activities...',
                placeholderValue: 'Choose an activity',
                noResultsText: 'No activities found',
                itemSelectText: '',
            });
        }
    }

    initializeDrinkChoicesSelectors() {
        // Destroy existing Choices instances if they exist
        if (this.peopleChoices) {
            this.peopleChoices.destroy();
            this.peopleChoices = null;
        }
        if (this.drinkChoices) {
            this.drinkChoices.destroy();
            this.drinkChoices = null;
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
        
        // Initialize drinks selector (multi-select with search)
        const drinkSelect = this.modalBody.querySelector('select[name="drinks"]');
        if (drinkSelect) {
            this.drinkChoices = new Choices(drinkSelect, {
                removeItemButton: true,
                searchEnabled: true,
                searchPlaceholderValue: 'Search drinks...',
                placeholderValue: 'Choose drinks',
                noResultsText: 'No drinks found',
                itemSelectText: '',
            });
        }
    }

    closeModal() {
        // Destroy Choices instances when closing modal
        if (this.peopleChoices) {
            this.peopleChoices.destroy();
            this.peopleChoices = null;
        }
        if (this.foodSourceChoices) {
            this.foodSourceChoices.destroy();
            this.foodSourceChoices = null;
        }
        if (this.activityChoices) {
            this.activityChoices.destroy();
            this.activityChoices = null;
        }
        if (this.drinkChoices) {
            this.drinkChoices.destroy();
            this.drinkChoices = null;
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