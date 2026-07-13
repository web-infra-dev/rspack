new Worker(
	/* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url)
);
