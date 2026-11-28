module.exports = function (source) {
	const [imports, topLevel, ...body] = source.split("\n");
	this.callback(null, [imports, topLevel, "(() => {", ...body, "})();"].join("\n"), {
		version: 3,
		sources: [this.resourcePath],
		sourcesContent: [source],
		names: [],
		mappings: "AAAA;AACA;;AACA;AACA;AACA"
	});
};
