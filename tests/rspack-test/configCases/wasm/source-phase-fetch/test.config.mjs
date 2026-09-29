import fs from "node:fs";
import path from "node:path";
import url from "node:url";

export default {
	findBundle(i) {
		switch (i) {
			case 0:
				return ["bundle0.mjs"];
			case 1:
				return ["chunks/694.async.js", "bundle1.js"];
		}
	},
	moduleScope(scope, _stats, options) {
		scope.fetch = resource =>
			new Promise((resolve, reject) => {
				const file = /^file:/i.test(resource)
					? url.fileURLToPath(resource)
					: path.join(options.output.path, path.basename(resource));

				fs.readFile(file, (err, data) => {
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
};
