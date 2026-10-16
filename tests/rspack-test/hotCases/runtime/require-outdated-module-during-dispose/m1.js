const state = require("./state");
require("./m2");

module.hot.dispose(() => {
	state.disposeRuns.push("./m1.js");
	const fresh = require("./m2");
	state.actives.push(() => fresh.getActive());
});

module.exports.getActive = () => module.hot.active;
