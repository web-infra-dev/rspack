const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: [
		async ({ context, getResolve }) => {
			const resolve = getResolve();
			expect(await resolve(context, "./index.js?foo=1#bar=1")).toBe(
				path.join(__dirname, "./index.js") + "?foo=1#bar=1"
			);
			expect(
				await new Promise((promiseResolve, promiseReject) => {
					resolve(context, "./index.js?foo=1#bar=1", (err, result) => {
						if (err) {
							promiseReject(err);
							return;
						}
						promiseResolve(result);
					});
				})
			).toBe(path.join(__dirname, "./index.js") + "?foo=1#bar=1");
			return false;
		}
	]
};
