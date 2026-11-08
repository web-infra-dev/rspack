export function createWorker() {
	return new Worker(
		/* webpackChunkName: "worker-facade" */ new URL(
			"./worker.js",
			import.meta["url"]
		)
	);
}
