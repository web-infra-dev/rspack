import { Worker } from "node:worker_threads";

it("should preserve every duplicated worker entry name", async () => {
	const workers = [
		new Worker(
			new URL(
				/* webpackChunkName: "named-worker-a" */ "./lib.js",
				import.meta.url
			)
		),
		new Worker(new URL("./lib.js", import.meta.url), {
			name: "named-worker-b"
		})
	];
	let results;

	try {
		results = await Promise.all(
			workers.map(
				worker =>
					new Promise((resolve, reject) => {
						worker.on("message", resolve);
						worker.on("error", reject);
						worker.on("exit", code => {
							reject(
								new Error(
									`Worker stopped before responding with exit code ${code}`
								)
							);
						});
					})
			)
		);
	} finally {
		await Promise.all(workers.map(worker => worker.terminate()));
	}

	expect(results).toEqual(["worker result", "worker result"]);
});
