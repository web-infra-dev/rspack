module.exports = {
	// The tests live in `a` only.
	findBundle: index => (index === 0 ? "./bundle.js" : null),
	moduleScope(scope) {
		const config = require("./rspack.config");

		scope.EVENTS = config.events;
		scope.STATE = config.state;
	}
};
