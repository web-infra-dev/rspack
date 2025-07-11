// Pure CommonJS module.exports patterns

// Simple object export
module.exports = {
	// Basic functions
	add: function (a, b) {
		return a + b;
	},

	subtract: function (a, b) {
		return a - b;
	},

	multiply: (a, b) => a * b,

	// Object properties
	constants: {
		PI: Math.PI,
		E: Math.E,
		GOLDEN_RATIO: 1.61803
	},

	// Nested object with methods
	math: {
		square: function (n) {
			return n * n;
		},
		cube: function (n) {
			return n * n * n;
		},
		factorial: function (n) {
			if (n <= 1) return 1;
			return n * this.factorial(n - 1);
		}
	},

	// Factory function
	createCalculator: function (initialValue = 0) {
		return {
			value: initialValue,
			add: function (n) {
				this.value += n;
				return this;
			},
			multiply: function (n) {
				this.value *= n;
				return this;
			},
			result: function () {
				return this.value;
			},
			reset: function () {
				this.value = 0;
				return this;
			}
		};
	},

	// Async function
	asyncOperation: async function (delay = 1000) {
		return new Promise(resolve => {
			setTimeout(() => resolve(`Completed after ${delay}ms`), delay);
		});
	},

	// Unused exports for testing
	unusedMethod: function () {
		return "This method is not used";
	},

	unusedProperty: "This property is not used",

	unusedObject: {
		nested: {
			property: "unused"
		}
	}
};
