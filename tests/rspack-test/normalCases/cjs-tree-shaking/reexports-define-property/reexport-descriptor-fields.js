Object.defineProperty(exports, "eager", {
	enumerable: false,
	configurable: true,
	writable: true,
	value: require("./module?descriptor-eager").abc
});

Object.defineProperty(exports, "lazy", {
	enumerable: false,
	configurable: true,
	get: async () => require("./module?descriptor-lazy").def
});
