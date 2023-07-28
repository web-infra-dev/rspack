const reactRefresh = require("@rspack/dev-client/react-refresh");

function shouldLooksLikeAModuleId(id) {
	console.log(id);
	if (typeof id === "string" && !id.includes("[object Object]")) {
		return;
	}
	throw new Error(`Looks like ${id} is not a module.id`);
}

module.exports = {
	...reactRefresh,
	register(type, id) {
		shouldLooksLikeAModuleId(id);
		return reactRefresh.register(type, id);
	}
};
