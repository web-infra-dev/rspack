const state = require("./state");

module.hot.dispose(() => {
	const fresh = require("./m1");
	state.actives.push(() => fresh.getActive());
});

module.exports.getActive = () => module.hot.active;
