// Main CommonJS module entry point
const { validateEmail, formatPhoneNumber } = require('./utils');
const { deepClone, mergeObjects } = require('./helpers');

// Regular CommonJS exports
exports.validateEmail = validateEmail;
exports.formatPhoneNumber = formatPhoneNumber;
exports.deepClone = deepClone;
exports.mergeObjects = mergeObjects;

// Direct function exports
exports.capitalize = function(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
};

exports.slugify = function(str) {
  return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
};

// Object export
exports.constants = {
  MAX_RETRY_COUNT: 3,
  DEFAULT_TIMEOUT: 5000,
  API_VERSION: 'v1'
};

// Utility functions
exports.debounce = function(func, wait) {
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

// Factory function
exports.createLogger = function(prefix = 'LOG') {
  return {
    info: (msg) => console.log(`[${prefix}:INFO] ${msg}`),
    warn: (msg) => console.warn(`[${prefix}:WARN] ${msg}`),
    error: (msg) => console.error(`[${prefix}:ERROR] ${msg}`)
  };
};

// Unused exports for testing tree-shaking
exports.unusedFunction = function() {
  return "This should appear in unused exports";
};

exports.unusedConstant = "UNUSED_VALUE";

exports.unusedObject = {
  prop: "unused property"
};