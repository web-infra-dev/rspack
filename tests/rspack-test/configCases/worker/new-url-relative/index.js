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

new SharedWorker(
	/* webpackChunkName: "shared-string-literal" */ new URL(
		"./worker.js",
		import.meta.url
	),
	"string-literal"
);

new SharedWorker(new URL("./worker.js", import.meta.url), "chat");

new SharedWorker(
	/* webpackChunkName: "shared-object-literal" */ new URL(
		"./worker.js",
		import.meta.url
	),
	{ name: "object-literal", type: "classic" }
);

const sharedWorkerName = "string-variable";
new SharedWorker(
	/* webpackChunkName: "shared-string-variable" */ new URL(
		"./worker.js",
		import.meta.url
	),
	sharedWorkerName
);

const sharedWorkerOptions = {
	name: "object-variable",
	type: "classic"
};
new SharedWorker(
	/* webpackChunkName: "shared-object-variable" */ new URL(
		"./worker.js",
		import.meta.url
	),
	sharedWorkerOptions
);

let sharedWorkerOptionsEvaluationCount = 0;
const getSharedWorkerOptions = () => {
	sharedWorkerOptionsEvaluationCount++;
	return "string-expression";
};
new SharedWorker(
	/* webpackChunkName: "shared-string-expression" */ new URL(
		"./worker.js",
		import.meta.url
	),
	getSharedWorkerOptions()
);
globalThis.__sharedWorkerOptionsEvaluationCount =
	sharedWorkerOptionsEvaluationCount;
