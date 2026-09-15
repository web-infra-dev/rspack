"use strict";

// Skip if WebAssembly or Response is not supported
module.exports = () => {
	try {
		return (
			typeof WebAssembly !== "undefined" &&
			WebAssembly.Module !== undefined &&
			typeof Response !== "undefined"
		);
	} catch (e) {
		return false;
	}
};
