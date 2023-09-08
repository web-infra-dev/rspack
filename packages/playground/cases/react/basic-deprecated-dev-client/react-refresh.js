const reactRefresh = require("@rspack/plugin-react-refresh/react-refresh");

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
