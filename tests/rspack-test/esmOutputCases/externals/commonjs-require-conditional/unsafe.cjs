const selected = require(
	globalThis.__USE_EXTERNAL__ ? "external" : "./local.cjs"
);

const contextual = globalThis.__RUN_DYNAMIC_CONTEXT__
	? require(
			globalThis.__USE_CONTEXT_EXTERNAL__
				? "external"
				: "./" + globalThis.__LOCAL_MODULE__
		)
	: require("./local.cjs");

module.exports = { selected, contextual };
