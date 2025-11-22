const React = require("react");

// top-level sync consume of shared React to mimic real-world usage
module.exports = function Widget({ who = "world" } = {}) {
	return React.createElement("div", null, `Remote Widget says hi to ${who}`);
};
