// Shared utility functions - testing various export scenarios

// Used export (imported in index.js)
export const formatDate = (date) => {
  return new Intl.DateTimeFormat('en-US').format(date);
};

// Used export (imported in index.js)
export const capitalize = (str) => {
  return str.charAt(0).toUpperCase() + str.slice(1);
};

// Unused export (not imported anywhere)
export const debounce = (func, wait) => {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
};

// Additional unused exports for testing
export const toLowerCase = (str) => str.toLowerCase();
export const padString = (str, length, char = ' ') => str.padStart(length, char);

// Used default export (not imported but defined)
export default {
  formatDate,
  capitalize,
  debounce,
  toLowerCase,
  padString
};