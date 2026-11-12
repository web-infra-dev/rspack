import fs from "node:fs";
import url from "node:url";
import worker from "node:worker_threads";

export default {
	moduleScope(scope, _stats, options) {
		if (options.name.includes("node")) {
			delete scope.window;
			delete scope.document;
			delete scope.self;
			scope.Worker = worker.Worker;
		} else {
			scope.fetch = (resource) =>
				new Promise((resolve, reject) => {
					fs.readFile(url.fileURLToPath(resource), (err, data) => {
						if (err) {
							reject(err);
							return;
						}

						return resolve(
							new Response(data, {
								headers: { "Content-Type": "application/wasm" }
							})
						);
					});
				});
		}
	}
};
