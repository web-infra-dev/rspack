"use strict";

// Skip if WebAssembly is not supported
module.exports = () => {
	try {
		return typeof WebAssembly !== "undefined" && WebAssembly.Module !== undefined;
	} catch (e) {
		return false;
	}
};
