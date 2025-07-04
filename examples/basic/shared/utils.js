import { DEFAULT_TIMEOUT } from "./config.js";
// Shared utility functions
import { deepClone, generateId, validateEmail } from "./nested-utils.js";

// Import CommonJS helper to test PURE annotations for CommonJS requires
const cjsHelper = require("./cjs-helper.js");

export const formatDate = date => {
	return new Intl.DateTimeFormat("en-US").format(date);
};

export const capitalize = str => {
	return str.charAt(0).toUpperCase() + str.slice(1);
};

// Use CommonJS helper function to test CommonJS integration
export const processWithHelper = input => {
	return cjsHelper.helperFunction(input);
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
