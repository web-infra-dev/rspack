// Skip if WebAssembly is not supported
export default () => {
	try {
		return typeof WebAssembly !== "undefined" && WebAssembly.Module !== undefined;
	} catch (e) {
		return false;
	}
};
