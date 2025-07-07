// Mixed export module with ES6 and CommonJS patterns

// ES6 named exports
export const version = "1.0.0";
export const config = {
	debug: false,
	environment: "development"
};

// ES6 function exports
export function calculate(a, b, operation = "add") {
	switch (operation) {
		case "add":
			return a + b;
		case "subtract":
			return a - b;
		case "multiply":
			return a * b;
		case "divide":
			return b !== 0 ? a / b : Number.NaN;
		default:
			return Number.NaN;
	}
}

export function validateInput(input) {
	return typeof input === "string" && input.length > 0;
}

// ES6 class export
export class DataProcessor {
	constructor(name) {
		this.name = name;
		this.data = [];
	}

	add(item) {
		this.data.push(item);
		return this;
	}

	process() {
		return this.data.map(item => ({
			processed: true,
			value: item,
			timestamp: Date.now()
		}));
	}

	clear() {
		this.data = [];
		return this;
	}
}

// ES6 arrow function export
export const createLogger = (prefix = "LOG") => {
	return {
		info: msg => console.log(`[${prefix}:INFO] ${msg}`),
		warn: msg => console.warn(`[${prefix}:WARN] ${msg}`),
		error: msg => console.error(`[${prefix}:ERROR] ${msg}`)
	};
};

// CommonJS style exports (mixed)
module.exports.legacy = {
	oldFunction: function () {
		return "legacy function";
	},
	CONSTANT: 42
};

// Unused exports for testing tree-shaking
export function unusedFunction() {
	return "This should appear in unused exports";
}

export const unusedConstant = "unused";

export class UnusedClass {
	constructor() {
		this.unused = true;
	}
}

// Default export
const mixedExports = {
	version,
	config,
	calculate,
	validateInput,
	DataProcessor,
	createLogger
};

export default mixedExports;
