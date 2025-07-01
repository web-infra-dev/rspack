// CommonJS module using pure module.exports = { ... } pattern
// This demonstrates the classic CommonJS export style where everything is exported as a single object

// Utility functions defined locally
function calculateSum(numbers) {
	return numbers.reduce((acc, num) => acc + num, 0);
}

function calculateAverage(numbers) {
	if (numbers.length === 0) return 0;
	return calculateSum(numbers) / numbers.length;
}

function findMinMax(numbers) {
	if (numbers.length === 0) return { min: null, max: null };
	return {
		min: Math.min(...numbers),
		max: Math.max(...numbers)
	};
}

function formatCurrency(amount, currency = "USD") {
	return new Intl.NumberFormat("en-US", {
		style: "currency",
		currency: currency
	}).format(amount);
}

function formatPercentage(value, decimals = 1) {
	return `${(value * 100).toFixed(decimals)}%`;
}

// Data processing utilities
function transformData(data, transformer) {
	if (!Array.isArray(data)) return [];
	return data.map(transformer);
}

function filterData(data, predicate) {
	if (!Array.isArray(data)) return [];
	return data.filter(predicate);
}

function groupBy(array, keyFunction) {
	return array.reduce((groups, item) => {
		const key = keyFunction(item);
		if (!groups[key]) {
			groups[key] = [];
		}
		groups[key].push(item);
		return groups;
	}, {});
}

// String utilities
function slugify(text) {
	return text
		.toLowerCase()
		.trim()
		.replace(/\s+/g, "-")
		.replace(/[^\w\-]+/g, "");
}

function capitalize(text) {
	return text.charAt(0).toUpperCase() + text.slice(1);
}

function truncate(text, maxLength, suffix = "...") {
	if (text.length <= maxLength) return text;
	return text.substring(0, maxLength - suffix.length) + suffix;
}

// Date utilities
function formatDate(date, format = "short") {
	const options = {
		short: { year: "numeric", month: "short", day: "numeric" },
		long: { year: "numeric", month: "long", day: "numeric", weekday: "long" },
		iso: undefined // Will use toISOString
	};

	if (format === "iso") {
		return date.toISOString();
	}

	return new Intl.DateTimeFormat(
		"en-US",
		options[format] || options.short
	).format(date);
}

function isWeekend(date) {
	const day = date.getDay();
	return day === 0 || day === 6; // Sunday or Saturday
}

// Validation utilities
function isEmail(email) {
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	return emailRegex.test(email);
}

function isUrl(url) {
	try {
		new URL(url);
		return true;
	} catch {
		return false;
	}
}

function isEmpty(value) {
	if (value == null) return true;
	if (typeof value === "string") return value.trim().length === 0;
	if (Array.isArray(value)) return value.length === 0;
	if (typeof value === "object") return Object.keys(value).length === 0;
	return false;
}

// Constants
const MATH_CONSTANTS = {
	PI: Math.PI,
	E: Math.E,
	GOLDEN_RATIO: (1 + Math.sqrt(5)) / 2,
	EULER_MASCHERONI: 0.5772156649015329
};

const HTTP_STATUS = {
	OK: 200,
	CREATED: 201,
	BAD_REQUEST: 400,
	UNAUTHORIZED: 401,
	FORBIDDEN: 403,
	NOT_FOUND: 404,
	INTERNAL_SERVER_ERROR: 500
};

// Complex utility class
class DataStore {
	constructor() {
		this.data = new Map();
		this.listeners = new Set();
	}

	set(key, value) {
		const oldValue = this.data.get(key);
		this.data.set(key, value);
		this.notifyListeners("set", { key, value, oldValue });
		return this;
	}

	get(key) {
		return this.data.get(key);
	}

	has(key) {
		return this.data.has(key);
	}

	delete(key) {
		const existed = this.data.has(key);
		const result = this.data.delete(key);
		if (existed) {
			this.notifyListeners("delete", { key });
		}
		return result;
	}

	clear() {
		this.data.clear();
		this.notifyListeners("clear", {});
	}

	subscribe(listener) {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}

	notifyListeners(event, data) {
		for (const listener of this.listeners) {
			try {
				listener(event, data);
			} catch (error) {
				console.error("DataStore listener error:", error);
			}
		}
	}

	toJSON() {
		return Object.fromEntries(this.data);
	}
}

// Export everything using the classic module.exports = { ... } pattern
// This is the pure CommonJS style where all exports are defined in a single object
module.exports = {
	// Math utilities
	calculateSum,
	calculateAverage,
	findMinMax,

	// Formatting utilities
	formatCurrency,
	formatPercentage,

	// Data processing
	transformData,
	filterData,
	groupBy,

	// String utilities
	slugify,
	capitalize,
	truncate,

	// Date utilities
	formatDate,
	isWeekend,

	// Validation utilities
	isEmail,
	isUrl,
	isEmpty,

	// Constants
	MATH_CONSTANTS,
	HTTP_STATUS,

	// Class constructor
	DataStore,

	// Factory function for DataStore
	createDataStore: () => new DataStore(),

	// Meta information
	moduleInfo: {
		name: "module-exports-pattern",
		version: "1.0.0",
		type: "commonjs-pure-exports",
		description: "Pure module.exports pattern demonstration",
		exportCount: 19, // Total number of exports
		exportTypes: ["function", "object", "class", "constant"]
	}
};
