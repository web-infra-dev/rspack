// CommonJS module using module.exports = { ... } pattern

function calculateSum(numbers) {
	return numbers.reduce((a, b) => a + b, 0);
}

function calculateAverage(numbers) {
	return calculateSum(numbers) / numbers.length;
}

function formatCurrency(amount) {
	return `$${amount.toFixed(2)}`;
}

function formatPercentage(value) {
	return `${(value * 100).toFixed(1)}%`;
}

const helpers = {
	isPositive: n => n > 0,
	isNegative: n => n < 0
};

// Export using module.exports = { ... } pattern
module.exports = {
	calculateSum,
	calculateAverage,
	formatCurrency,
	formatPercentage,
	helpers,
	// Direct property definition
	VERSION: "1.0.0",
	// Nested object
	config: {
		locale: "en-US",
		precision: 2
	}
};
