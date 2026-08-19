const value = require("external-value");

if (value !== 42) {
	throw new Error("external request rest segments should keep their wrapper semantics");
}

export { value };
