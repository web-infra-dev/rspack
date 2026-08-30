Object.defineProperty(exports, "invalid", {
	get: () => require("./module?invalid-descriptor").abc,
	writable: true
});
