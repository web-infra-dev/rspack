// CommonJS module with mixed export patterns

// Pattern 1: Direct property assignment
exports.utilityA = function (x) {
	return x + 1;
};

// Pattern 2: module.exports property assignment
module.exports.utilityB = function (x) {
	return x * 2;
};

// Pattern 3: Object.defineProperty
Object.defineProperty(exports, "utilityC", {
	value: function (x) {
		return x - 1;
	},
	enumerable: true
});

// Pattern 4: Destructured assignment (less common)
const utilityD = function (x) {
	return x / 2;
};
const utilityE = function (x) {
	return x ** 2;
};

Object.assign(exports, {
	utilityD,
	utilityE
});

// Some unused exports
exports.unusedUtilityF = function () {
	return "Not used";
};

module.exports.unusedUtilityG = function () {
	return "Also not used";
};
