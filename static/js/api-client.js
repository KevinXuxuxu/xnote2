/**
 * API Client for XNote REST endpoints
 */
class ApiClient {
    constructor(baseUrl = '/api/v1') {
        this.baseUrl = baseUrl;
    }

    /**
     * Generic HTTP request handler
     */
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;

        const config = {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        };

        try {
            const response = await fetch(url, config);

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error('API request failed:', error);
            throw error;
        }
    }

    // Meals API
    async getMeals() {
        return this.request('/meals');
    }

    async getMeal(id) {
        return this.request(`/meals/${id}`);
    }

    async getMealDetails(id) {
        return this.request(`/meals/${id}/details`);
    }

    async createMeal(meal) {
        return this.request('/meals', {
            method: 'POST',
            body: JSON.stringify(meal)
        });
    }

    async updateMeal(id, meal) {
        return this.request(`/meals/${id}`, {
            method: 'PUT',
            body: JSON.stringify(meal)
        });
    }

    async deleteMeal(id) {
        return this.request(`/meals/${id}`, {
            method: 'DELETE'
        });
    }

    async deleteMealsBatch(mealIds) {
        return this.request('/meals/batch/delete', {
            method: 'POST',
            body: JSON.stringify({ meal_ids: mealIds })
        });
    }

    // Events API
    async getEvents() {
        return this.request('/events');
    }

    async getEvent(id) {
        return this.request(`/events/${id}`);
    }

    async getEventDetails(id) {
        return this.request(`/events/${id}/details`);
    }

    async createEvent(event) {
        return this.request('/events', {
            method: 'POST',
            body: JSON.stringify(event)
        });
    }

    async updateEvent(id, event) {
        return this.request(`/events/${id}`, {
            method: 'PUT',
            body: JSON.stringify(event)
        });
    }

    async deleteEvent(id) {
        return this.request(`/events/${id}`, {
            method: 'DELETE'
        });
    }

    // Drinks API
    async getDrinks() {
        return this.request('/drinks');
    }

    async getDrink(id) {
        return this.request(`/drinks/${id}`);
    }

    async getDrinkDetails(id) {
        return this.request(`/drinks/${id}/details`);
    }

    async createDrink(drink) {
        return this.request('/drinks', {
            method: 'POST',
            body: JSON.stringify(drink)
        });
    }

    async updateDrink(id, drink) {
        return this.request(`/drinks/${id}`, {
            method: 'PUT',
            body: JSON.stringify(drink)
        });
    }

    async deleteDrink(id) {
        return this.request(`/drinks/${id}`, {
            method: 'DELETE'
        });
    }

    // People API
    async getPeople() {
        return this.request('/people');
    }

    async getPerson(id) {
        return this.request(`/people/${id}`);
    }

    async createPerson(person) {
        return this.request('/people', {
            method: 'POST',
            body: JSON.stringify(person)
        });
    }

    async updatePerson(id, person) {
        return this.request(`/people/${id}`, {
            method: 'PUT',
            body: JSON.stringify(person)
        });
    }

    async deletePerson(id) {
        return this.request(`/people/${id}`, {
            method: 'DELETE'
        });
    }

    // Restaurants API
    async getRestaurants() {
        return this.request('/restaurants');
    }

    async getRestaurant(id) {
        return this.request(`/restaurants/${id}`);
    }

    async createRestaurant(restaurant) {
        return this.request('/restaurants', {
            method: 'POST',
            body: JSON.stringify(restaurant)
        });
    }

    async updateRestaurant(id, restaurant) {
        return this.request(`/restaurants/${id}`, {
            method: 'PUT',
            body: JSON.stringify(restaurant)
        });
    }

    async deleteRestaurant(id) {
        return this.request(`/restaurants/${id}`, {
            method: 'DELETE'
        });
    }

    // Recipes API
    async getRecipes() {
        return this.request('/recipes');
    }

    async getRecipe(id) {
        return this.request(`/recipes/${id}`);
    }

    async createRecipe(recipe) {
        return this.request('/recipes', {
            method: 'POST',
            body: JSON.stringify(recipe)
        });
    }

    async updateRecipe(id, recipe) {
        return this.request(`/recipes/${id}`, {
            method: 'PUT',
            body: JSON.stringify(recipe)
        });
    }

    async deleteRecipe(id) {
        return this.request(`/recipes/${id}`, {
            method: 'DELETE'
        });
    }

    // Products API
    async getProducts() {
        return this.request('/products');
    }

    async getProduct(id) {
        return this.request(`/products/${id}`);
    }

    async createProduct(product) {
        return this.request('/products', {
            method: 'POST',
            body: JSON.stringify(product)
        });
    }

    async updateProduct(id, product) {
        return this.request(`/products/${id}`, {
            method: 'PUT',
            body: JSON.stringify(product)
        });
    }

    async deleteProduct(id) {
        return this.request(`/products/${id}`, {
            method: 'DELETE'
        });
    }

    // Activities API
    async getActivities() {
        return this.request('/activities');
    }

    async getActivity(id) {
        return this.request(`/activities/${id}`);
    }

    async createActivity(activity) {
        return this.request('/activities', {
            method: 'POST',
            body: JSON.stringify(activity)
        });
    }

    async updateActivity(id, activity) {
        return this.request(`/activities/${id}`, {
            method: 'PUT',
            body: JSON.stringify(activity)
        });
    }

    async deleteActivity(id) {
        return this.request(`/activities/${id}`, {
            method: 'DELETE'
        });
    }

    // Locations API (enum-like)
    async getLocations() {
        return this.request('/locations');
    }

    async createLocation(location) {
        return this.request('/locations', {
            method: 'POST',
            body: JSON.stringify(location)
        });
    }

    async deleteLocation(name) {
        return this.request(`/locations/${encodeURIComponent(name)}`, {
            method: 'DELETE'
        });
    }

    // Food Types API (enum-like)
    async getFoodTypes() {
        return this.request('/food-types');
    }

    async createFoodType(foodType) {
        return this.request('/food-types', {
            method: 'POST',
            body: JSON.stringify(foodType)
        });
    }

    async deleteFoodType(name) {
        return this.request(`/food-types/${encodeURIComponent(name)}`, {
            method: 'DELETE'
        });
    }

    // Drink Options API (enum-like)
    async getDrinkOptions() {
        return this.request('/drink-options');
    }

    async createDrinkOption(drinkOption) {
        return this.request('/drink-options', {
            method: 'POST',
            body: JSON.stringify(drinkOption)
        });
    }

    async deleteDrinkOption(name) {
        return this.request(`/drink-options/${encodeURIComponent(name)}`, {
            method: 'DELETE'
        });
    }

    // Activity Types API (enum-like)
    async getActivityTypes() {
        return this.request('/activity-types');
    }

    async createActivityType(activityType) {
        return this.request('/activity-types', {
            method: 'POST',
            body: JSON.stringify(activityType)
        });
    }

    async deleteActivityType(name) {
        return this.request(`/activity-types/${encodeURIComponent(name)}`, {
            method: 'DELETE'
        });
    }

    // Daily Summary API
    async getDailySummary(startDate = null, endDate = null) {
        let endpoint = '/daily-summary';
        const params = new URLSearchParams();

        if (startDate) {
            params.append('start_date', startDate);
        }
        if (endDate) {
            params.append('end_date', endDate);
        }

        if (params.toString()) {
            endpoint += '?' + params.toString();
        }

        return this.request(endpoint);
    }

    // Utility methods for aggregated data
    async getAllEvents() {
        try {
            const [meals, events, drinks] = await Promise.all([
                this.getMeals(),
                this.getEvents(),
                this.getDrinks()
            ]);

            // Combine all events with type information
            const allEvents = [
                ...meals.map(meal => ({ ...meal, eventType: 'meal' })),
                ...events.map(event => ({ ...event, eventType: 'event' })),
                ...drinks.map(drink => ({ ...drink, eventType: 'drink' }))
            ];

            // Sort by date descending
            return allEvents.sort((a, b) => new Date(b.date) - new Date(a.date));
        } catch (error) {
            console.error('Failed to fetch all events:', error);
            throw error;
        }
    }

    async getEventDetails(id, type) {
        switch (type) {
            case 'meal':
                return this.getMealDetails(id);
            case 'event':
                return this.getEventDetailsInternal(id);
            case 'drink':
                return this.getDrinkDetails(id);
            default:
                throw new Error(`Unknown event type: ${type}`);
        }
    }

    // Internal method to get event details without type parameter
    async getEventDetailsInternal(id) {
        return this.request(`/events/${id}/details`);
    }
}

// Create global API client instance
window.apiClient = new ApiClient();