export default () =>
	typeof WebAssembly !== "undefined" &&
	typeof Response !== "undefined" &&
	typeof WebAssembly.instantiateStreaming === "function";
