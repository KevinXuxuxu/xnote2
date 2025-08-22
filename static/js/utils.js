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