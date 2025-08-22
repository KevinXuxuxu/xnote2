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
        this.modalTitle.textContent = 'Add Recipe';
        this.renderForm();
        this.modal.style.display = 'block';
    }

    renderForm() {
        this.modalBody.innerHTML = `
            <form id="recipeForm">
                <div class="form-group">
                    <label for="recipeName">Name <span style="color: red;">*</span></label>
                    <input type="text" id="recipeName" name="name" required placeholder="Enter recipe name">
                </div>
                
                <div class="form-group">
                    <label for="recipeIngredients">Ingredients <span style="color: red;">*</span></label>
                    <textarea id="recipeIngredients" name="ingredients" required rows="4" 
                              placeholder="List ingredients (one per line or comma-separated)"></textarea>
                </div>
                
                <div class="form-group">
                    <label for="recipeProcedure">Procedure <span style="color: red;">*</span></label>
                    <textarea id="recipeProcedure" name="procedure" required rows="6" 
                              placeholder="Describe cooking steps"></textarea>
                </div>
                
                <div class="form-group">
                    <label for="recipeCautions">Cautions (Optional)</label>
                    <textarea id="recipeCautions" name="cautions" rows="3" 
                              placeholder="Any warnings, allergies, or special notes"></textarea>
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

    async saveRecipe(addAnother = false) {
        try {
            const formData = this.collectFormData();
            const errors = this.validateForm(formData);

            if (errors.length > 0) {
                alert('Please fix the following errors:\n' + errors.join('\n'));
                return;
            }

            await apiClient.createRecipe(formData);

            // Refresh the recipe spreadsheet
            if (window.recipeSpreadsheet) {
                window.recipeSpreadsheet.refresh();
            }

            if (addAnother) {
                // Keep modal open and reset form
                this.renderForm();
            } else {
                // Close modal
                this.closeModal();
            }

        } catch (error) {
            console.error('Failed to create recipe:', error);
            alert(`Failed to create recipe: ${error.message}`);
        }
    }

    closeModal() {
        this.modal.style.display = 'none';
    }
}

// Make RecipeForm globally available
window.RecipeForm = RecipeForm;