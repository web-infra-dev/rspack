// CommonJS module demonstrating different export patterns (browser-compatible)
// Simulated util module for browser environment
const util = {
	inspect: (obj, options) => {
		// Simplified inspect for browser
		return JSON.stringify(obj, null, options?.depth || 2);
	}
};

// Simple function exports
function processArray(arr, callback) {
	return arr.map(callback);
}

function filterArray(arr, predicate) {
	return arr.filter(predicate);
}

function reduceArray(arr, reducer, initial) {
	return arr.reduce(reducer, initial);
}

// Object with methods
const dataUtils = {
	sum: numbers => numbers.reduce((a, b) => a + b, 0),
	average: numbers =>
		numbers.length ? dataUtils.sum(numbers) / numbers.length : 0,
	max: numbers => Math.max(...numbers),
	min: numbers => Math.min(...numbers),

	// Nested object
	formatters: {
		currency: amount => `$${amount.toFixed(2)}`,
		percentage: value => `${(value * 100).toFixed(1)}%`,
		number: value => value.toLocaleString()
	}
};

// Class definition
class DataProcessor {
	constructor(options = {}) {
		this.options = {
			debug: false,
			maxItems: 1000,
			...options
		};
	}

	process(data) {
		if (this.options.debug) {
			console.log("Processing data:", util.inspect(data, { depth: 2 }));
		}

		if (Array.isArray(data)) {
			return data.slice(0, this.options.maxItems);
		}

		return data;
	}

	validate(data) {
		return data != null && (Array.isArray(data) || typeof data === "object");
	}
}

// Export individual functions
exports.processArray = processArray;
exports.filterArray = filterArray;
exports.reduceArray = reduceArray;

// Export object
exports.dataUtils = dataUtils;

// Export class
exports.DataProcessor = DataProcessor;

// Export constants
exports.DEFAULT_OPTIONS = {
	debug: false,
	maxItems: 1000,
	timeout: 5000
};

// Export a factory function
exports.createProcessor = function (options) {
	return new DataProcessor(options);
};

// Mixed export pattern - also assign to module.exports
module.exports = {
	// Include all named exports
	...exports,

	// Add default export behavior
	default: dataUtils,

	// Meta information
	__esModule: false, // Explicitly CommonJS
	version: "1.0.0",
	type: "data-processor"
};
