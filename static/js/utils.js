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