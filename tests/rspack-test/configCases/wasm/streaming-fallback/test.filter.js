"use strict";

module.exports = () =>
	typeof WebAssembly !== "undefined" &&
	typeof Response !== "undefined" &&
	typeof WebAssembly.instantiateStreaming === "function";
