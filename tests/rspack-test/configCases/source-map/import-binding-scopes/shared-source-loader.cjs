const path = require("path");

module.exports = function (source) {
	this.callback(null, source.replace("./lib", "./alternate"), {
		version: 3,
		sources: [path.join(this.context, "shared-source.js")],
		sourcesContent: [source],
		names: [],
		mappings: "AAAA;AACA;AACA"
	});
};
