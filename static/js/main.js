/**
 * Main application initialization and event handlers
 */
document.addEventListener('DOMContentLoaded', function() {
    // Initialize components
    window.eventSpreadsheet = new EventSpreadsheet('main-spreadsheet');
    
    // Setup control button handlers
    setupControlButtons();
    
    // Setup filter handlers
    setupFilters();
    
    // Set default date filters (last 30 days)
    setDefaultDateFilters();
});

function setupControlButtons() {
    // Add event buttons
    document.getElementById('addMealBtn').onclick = () => {
        window.detailForms.openDetails(null, 'meal');
    };
    
    document.getElementById('addEventBtn').onclick = () => {
        window.detailForms.openDetails(null, 'event');
    };
    
    document.getElementById('addDrinkBtn').onclick = () => {
        window.detailForms.openDetails(null, 'drink');
    };
    
    // Refresh button
    document.getElementById('refreshBtn').onclick = () => {
        window.eventSpreadsheet.refresh();
    };
}

function setupFilters() {
    document.getElementById('applyFilters').onclick = () => {
        const filters = {
            startDate: document.getElementById('startDate').value,
            endDate: document.getElementById('endDate').value
        };
        
        window.eventSpreadsheet.setFilters(filters);
    };
    
    // Auto-apply filters when date inputs change
    document.getElementById('startDate').onchange = autoApplyFilters;
    document.getElementById('endDate').onchange = autoApplyFilters;
}

function autoApplyFilters() {
    // Small delay to allow for rapid changes
    clearTimeout(window.filterTimeout);
    window.filterTimeout = setTimeout(() => {
        document.getElementById('applyFilters').click();
    }, 300);
}

function setDefaultDateFilters() {
    document.getElementById('startDate').value = window.dateUtils.getDaysAgoLocal(30);
    document.getElementById('endDate').value = window.dateUtils.getTodayLocal();
}

// Keyboard shortcuts
document.addEventListener('keydown', function(e) {
    // Ctrl/Cmd + N for new meal
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'meal');
    }
    
    // Ctrl/Cmd + E for new event
    if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'event');
    }
    
    // Ctrl/Cmd + D for new drink
    if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
        e.preventDefault();
        window.detailForms.openDetails(null, 'drink');
    }
    
    // F5 or Ctrl/Cmd + R for refresh
    if (e.key === 'F5' || ((e.ctrlKey || e.metaKey) && e.key === 'r')) {
        e.preventDefault();
        window.eventSpreadsheet.refresh();
    }
});

// Global error handler
window.addEventListener('error', function(e) {
    console.error('Global error:', e.error);
    
    // Show user-friendly error message
    const errorDiv = document.createElement('div');
    errorDiv.className = 'alert alert-error';
    errorDiv.style.position = 'fixed';
    errorDiv.style.top = '20px';
    errorDiv.style.right = '20px';
    errorDiv.style.zIndex = '10000';
    errorDiv.style.maxWidth = '400px';
    errorDiv.innerHTML = `
        <strong>Error:</strong> Something went wrong. Please refresh the page.
        <button onclick="this.parentElement.remove()" style="float: right; background: none; border: none; font-size: 1.2em; cursor: pointer;">&times;</button>
    `;
    
    document.body.appendChild(errorDiv);
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
        if (errorDiv.parentElement) {
            errorDiv.remove();
        }
    }, 5000);
});

// Service worker registration for offline capabilities (future enhancement)
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/static/sw.js').catch(e => {
        console.log('Service worker registration failed:', e);
    });
}

// Utility functions
window.utils = {
    formatDate: function(dateString) {
        return new Date(dateString).toLocaleDateString();
    },
    
    formatPeople: function(people) {
        if (!people || people.length === 0) return '';
        return people.map(p => p.name).join(', ');
    },
    
    debounce: function(func, wait) {
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
    
    showToast: function(message, type = 'info') {
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

console.log('XNote frontend initialized successfully!');