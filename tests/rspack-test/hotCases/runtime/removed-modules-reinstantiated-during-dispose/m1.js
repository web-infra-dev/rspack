const state = require("./state");
require("./m2");

module.hot.dispose(() => {
	const fresh = require("./m2");
	state.actives.push(() => fresh.getActive());
});

module.exports.getActive = () => module.hot.active;
