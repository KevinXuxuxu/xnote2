/**
 * Utility functions for date handling and other common operations
 */
window.dateUtils = {
    /**
     * Format a Date object to YYYY-MM-DD string in local timezone
     * Fixes timezone issues with toISOString()
     */
    formatLocalDate: function (date) {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        return `${year}-${month}-${day}`;
    },

    /**
     * Get today's date in local timezone as YYYY-MM-DD
     */
    getTodayLocal: function () {
        return this.formatLocalDate(new Date());
    },

    /**
     * Get date N days ago in local timezone as YYYY-MM-DD
     */
    getDaysAgoLocal: function (days) {
        const date = new Date();
        date.setDate(date.getDate() - days);
        return this.formatLocalDate(date);
    }
};

// General utility functions
window.utils = {
    /**
     * Format a date string for display
     */
    formatDate: function (dateString) {
        return new Date(dateString).toLocaleDateString();
    },

    /**
     * Format people names for display
     */
    formatPeople: function (people) {
        if (!people) return '';

        // Handle string format (comma-separated names) or array of objects
        let nameArray;
        if (typeof people === 'string') {
            nameArray = people.split(',').map(name => name.trim()).filter(name => name);
        } else if (Array.isArray(people)) {
            if (people.length === 0) return '';
            nameArray = people.map(p => p.name);
        } else {
            return '';
        }

        // Check if it's exactly just xx and ww - if so, omit them
        const lowerNames = nameArray.map(name => name.toLowerCase());
        if (lowerNames.length === 2 && lowerNames.includes('xx') && lowerNames.includes('ww')) {
            return '';
        }

        // Sort people: xx and ww first, then others alphabetically
        const sortedNames = nameArray.slice().sort((a, b) => {
            const aName = a.toLowerCase();
            const bName = b.toLowerCase();

            // xx comes first
            if (aName === 'xx') return -1;
            if (bName === 'xx') return 1;

            // ww comes second
            if (aName === 'ww') return -1;
            if (bName === 'ww') return 1;

            // Others alphabetically
            return aName.localeCompare(bName);
        });

        return sortedNames.join(', ');
    },

    /**
     * Debounce function to limit how often a function can be called
     */
    debounce: function (func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    },

    /**
     * Show toast notification
     */
    showToast: function (message, type = 'info') {
        const toast = document.createElement('div');
        toast.className = `alert alert-${type}`;
        toast.style.position = 'fixed';
        toast.style.top = '20px';
        toast.style.right = '20px';
        toast.style.zIndex = '10000';
        toast.style.maxWidth = '400px';
        toast.innerHTML = `
            ${message}
            <button onclick="this.parentElement.remove()" style="float: right; background: none; border: none; font-size: 1.2em; cursor: pointer;">&times;</button>
        `;

        document.body.appendChild(toast);

        // Auto-remove after 3 seconds
        setTimeout(() => {
            if (toast.parentElement) {
                toast.remove();
            }
        }, 3000);
    }
};

/**
 * Duplicate detection utilities
 */
window.duplicateUtils = {
    /**
     * Check for potential duplicates using substring matching
     * Returns true if newName is a substring of existingName or vice versa
     * @param {string} newName - The new name to check
     * @param {string} existingName - The existing name to compare against
     * @returns {boolean} True if there's a potential duplicate
     */
    isPotentialDuplicate: function (newName, existingName) {
        if (!newName || !existingName) return false;
        
        const cleanNew = newName.trim().toLowerCase();
        const cleanExisting = existingName.trim().toLowerCase();
        
        if (cleanNew === cleanExisting) return true;
        
        return cleanNew.includes(cleanExisting) || cleanExisting.includes(cleanNew);
    },

    /**
     * Find potential duplicates in an array of existing items
     * @param {string} newName - The new name to check
     * @param {Array} existingItems - Array of existing items (should have 'name' property)
     * @returns {Array} Array of potential duplicates
     */
    findPotentialDuplicates: function (newName, existingItems) {
        if (!newName || !existingItems || !Array.isArray(existingItems)) return [];
        
        const duplicates = [];
        
        for (const item of existingItems) {
            if (item.name && this.isPotentialDuplicate(newName, item.name)) {
                duplicates.push(item);
            }
        }
        
        return duplicates;
    },

    /**
     * Show confirmation dialog for potential duplicates
     * @param {string} newItemType - Type of item being created (e.g., 'Recipe', 'Person')
     * @param {string} newName - The new name to check
     * @param {Array} duplicates - Array of potential duplicates
     * @returns {boolean} True if user confirms creation
     */
    showDuplicateConfirmation: function (newItemType, newName, duplicates) {
        if (!duplicates || duplicates.length === 0) return true;
        
        const duplicateNames = duplicates.map(item => item.name).join('\n• ');
        
        const message = `Potential ${newItemType} duplicate detected!\n\n` +
                      `New name: "${newName}"\n\n` +
                      `Similar existing ${newItemType.toLowerCase()}(s):\n• ${duplicateNames}\n\n` +
                      `Do you want to continue creating this ${newItemType.toLowerCase()}?`;
        
        return confirm(message);
    }
};