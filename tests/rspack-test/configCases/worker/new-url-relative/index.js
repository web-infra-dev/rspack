new Worker(
	/* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url)
);

new Worker(
	/* webpackChunkName: "trailing-comma" */ new URL(
		"./worker.js",
		import.meta.url
	),
);

const classicOptions = { type: "classic" };

new Worker(
	/* webpackChunkName: "spread-options" */ new URL(
		"./worker.js",
		import.meta.url
	),
	{ type: "module", ...classicOptions }
);

new Worker(
	/* webpackChunkName: "duplicate-type" */ new URL(
		"./worker.js",
		import.meta.url
	),
	{ type: "module", type: "classic" }
);
