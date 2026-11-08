export function createWorker() {
	return new Promise((resolve, reject) => {
		require.ensure([], () => {
			require.ensure([], () => {
				resolve(new Worker(
					/* webpackChunkName: "worker-facade" */ new URL(
						"./worker.js",
						import.meta.url
					)
				));
			}, error => reject(error));
		}, error => reject(error));
	});
}
