// Pure CommonJS module with only require() usage - no ES6 imports
const crypto = {
	// Simulated crypto for browser
	createHash: () => ({
		update: () => ({ digest: () => "mock-hash" })
	})
};

// Pure CommonJS export patterns
exports.generateId = function () {
	return `id_${Math.random().toString(36).substr(2, 9)}`;
};

exports.hashString = function (input) {
	return crypto.createHash("md5").update(input).digest("hex");
};

exports.validateInput = function (input) {
	return input && typeof input === "string" && input.trim().length > 0;
};

exports.processData = function (data) {
	if (!Array.isArray(data)) {
		return null;
	}
	return data.map(item => ({
		id: this.generateId(),
		hash: this.hashString(String(item)),
		valid: this.validateInput(String(item))
	}));
};

// Utility object
exports.helpers = {
	timestamp: () => Date.now(),
	random: () => Math.random(),
	formatNumber: num => num.toLocaleString()
};

// Constants
exports.CONSTANTS = {
	MAX_LENGTH: 100,
	MIN_LENGTH: 1,
	DEFAULT_PREFIX: "cjs_",
	SUPPORTED_TYPES: ["string", "number", "boolean"]
};

// Class export
class DataValidator {
	constructor(options = {}) {
		this.options = {
			strict: false,
			allowEmpty: false,
			...options
		};
	}

	validate(data) {
		if (!data && !this.options.allowEmpty) {
			return false;
		}
		return this.options.strict ? this.strictValidate(data) : true;
	}

	strictValidate(data) {
		return exports.CONSTANTS.SUPPORTED_TYPES.includes(typeof data);
	}
}

exports.DataValidator = DataValidator;

// Factory function
exports.createValidator = function (options) {
	return new DataValidator(options);
};

// This module will NOT be imported via ES6 - only via require()
module.exports.info = {
	name: "pure-cjs-helper",
	version: "1.0.0",
	type: "pure-commonjs",
	description: "CommonJS module accessed only via require()"
};
