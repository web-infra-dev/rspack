// Test different export types for analysis
export const namedExport = "This is a named export";

export function functionExport() {
	return "This is a function export";
}

// CommonJS style exports
export const cjsExport = require("./lib.js");

// Module exports
export const moduleExport = {
	type: "module",
	value: "test value"
};

// Defined export
export const definedExport = Object.defineProperty({}, "prop", {
	value: "defined property",
	enumerable: true
});

// Default export
const defaultExport = {
	type: "default",
	message: "This is the default export"
};

export default defaultExport;
