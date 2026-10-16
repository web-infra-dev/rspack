const state = require("./state");

module.hot.dispose(() => {
	state.disposeRuns.push("./m2.js");
	const fresh = require("./m1");
	state.actives.push(() => fresh.getActive());
});

module.exports.getActive = () => module.hot.active;
