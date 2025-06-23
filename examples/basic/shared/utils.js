// Shared utility functions
import { validateEmail, generateId, deepClone } from './nested-utils.js';
import { DEFAULT_TIMEOUT } from './config.js';

export const formatDate = (date) => {
  return new Intl.DateTimeFormat('en-US').format(date);
};

export const capitalize = (str) => {
  return str.charAt(0).toUpperCase() + str.slice(1);
};

// Re-export nested utilities
export { validateEmail, generateId, deepClone };

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

export default {
  formatDate,
  capitalize,
  debounce
};