const path = require("path");
module.exports = {
	// workaround for resolving with ConfigCase.template for modules
	moduleScope(a) {
		a.__dirname = path.resolve(__dirname, "./dist");
		a.require = require;
	}
};
