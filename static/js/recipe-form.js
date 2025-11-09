/**
 * Recipe Form Component for managing recipe creation modal
 */
class RecipeForm {
    constructor() {
        this.modal = document.getElementById('recipeModal');
        this.modalTitle = document.getElementById('modalTitle');
        this.modalBody = document.getElementById('modalBody');
        this.saveBtn = document.getElementById('saveBtn');
        this.saveAndAddBtn = document.getElementById('saveAndAddBtn');
        this.cancelBtn = document.getElementById('cancelBtn');
        this.closeBtn = document.querySelector('.close');

        this.currentRecipeId = null;
        this.setupEventListeners();
    }

    setupEventListeners() {
        this.closeBtn.onclick = () => this.closeModal();
        this.cancelBtn.onclick = () => this.closeModal();
        this.saveBtn.onclick = () => this.saveRecipe(false);
        this.saveAndAddBtn.onclick = () => this.saveRecipe(true);

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
        this.currentRecipeId = null;
        this.modalTitle.textContent = 'Add Recipe';
        this.saveAndAddBtn.textContent = 'Save and add another';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    async openModalForEdit(recipeId) {
        try {
            this.currentRecipeId = recipeId;
            this.modalTitle.textContent = 'Edit Recipe';
            this.saveAndAddBtn.textContent = 'Save as new';
            
            // Load existing recipe data
            const recipe = await apiClient.getRecipe(recipeId);
            this.renderForm(recipe);
            this.modal.style.display = 'block';
        } catch (error) {
            console.error('Failed to load recipe for editing:', error);
            alert(`Failed to load recipe for editing: ${error.message}`);
        }
    }

    renderForm(recipeData = null) {
        const name = recipeData?.name || '';
        const ingredients = recipeData?.ingredients || '';
        const procedure = recipeData?.procedure || '';
        const cautions = recipeData?.cautions || '';

        this.modalBody.innerHTML = `
            <form id="recipeForm">
                <div class="form-group">
                    <label for="recipeName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="recipeName" name="name" required placeholder="Enter recipe name" value="${name}">
                </div>
                
                <div class="form-group">
                    <label for="recipeIngredients">Ingredients <span style="color: red;">*</span></label>
                    <textarea id="recipeIngredients" name="ingredients" required rows="4" 
                              placeholder="List ingredients (one per line or comma-separated)">${ingredients}</textarea>
                </div>
                
                <div class="form-group">
                    <label for="recipeProcedure">Procedure <span style="color: red;">*</span></label>
                    <textarea id="recipeProcedure" name="procedure" required rows="6" 
                              placeholder="Describe cooking steps">${procedure}</textarea>
                </div>
                
                <div class="form-group">
                    <label for="recipeCautions">Cautions (Optional)</label>
                    <textarea id="recipeCautions" name="cautions" rows="3" 
                              placeholder="Any warnings, allergies, or special notes">${cautions}</textarea>
                </div>
            </form>
        `;

        // Focus on the name field
        setTimeout(() => {
            document.getElementById('recipeName').focus();
        }, 100);
    }

    collectFormData() {
        const form = document.getElementById('recipeForm');
        const formData = new FormData(form);
        const data = {};

        for (const [key, value] of formData.entries()) {
            data[key] = value.trim();
        }

        // Convert empty cautions to null
        if (!data.cautions) {
            data.cautions = null;
        }

        return data;
    }

    validateForm(data) {
        const errors = [];

        if (!data.name) {
            errors.push('Recipe name is required');
        }
        if (!data.ingredients) {
            errors.push('Ingredients are required');
        }
        if (!data.procedure) {
            errors.push('Procedure is required');
        }

        return errors;
    }

    async saveRecipe(saveAsNew = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            const isEditing = this.currentRecipeId !== null;
            const shouldCreateNew = !isEditing || saveAsNew;

            if (shouldCreateNew) {
                // Check for potential duplicates only when creating new recipes
                const existingRecipes = await apiClient.getRecipes();
                const duplicates = window.duplicateUtils.findPotentialDuplicates(formData.name, existingRecipes);
                
                if (duplicates.length > 0) {
                    const confirmed = window.duplicateUtils.showDuplicateConfirmation('Recipe', formData.name, duplicates);
                    if (!confirmed) {
                        return; // User cancelled the creation
                    }
                }

                await apiClient.createRecipe(formData);
            } else {
                // Update existing recipe
                await apiClient.updateRecipe(this.currentRecipeId, formData);
            }

            // Refresh the recipe spreadsheet
            if (window.recipeSpreadsheet) {
                window.recipeSpreadsheet.refresh();
            }

            if (saveAsNew) {
                // Save as new: keep modal open and reset form with current data
                this.currentRecipeId = null;
                this.modalTitle.textContent = 'Add Recipe';
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
            console.error('Failed to save recipe:', error);
            alert(`Failed to save recipe: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
        this.currentRecipeId = null;
    }
}

// Make RecipeForm globally available
window.RecipeForm = RecipeForm;