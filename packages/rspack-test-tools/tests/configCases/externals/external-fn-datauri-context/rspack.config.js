const rspack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	externals: [
		async function ({ context, request }, callback) {
			// copy from https://github.com/webpack/enhanced-resolve/blob/fd3bde2b8eb18eb2138f3a9cadebcfee26922d6f/lib/getPaths.js#L12
			// user may use enhanced-resolve inside external function
			function getPaths(path) {
				if (path === "/") return { paths: ["/"], segments: [""] };
				const parts = path.split(/(.*?[\\/]+)/);
				const paths = [path];
				const segments = [parts[parts.length - 1]];
				let part = parts[parts.length - 1];
				path = path.substr(0, path.length - part.length - 1);
				for (let i = parts.length - 2; i > 2; i -= 2) {
					paths.push(path);
					part = parts[i];
					path = path.substr(0, path.length - part.length) || "/";
					segments.push(part.substr(0, part.length - 1));
				}
				part = parts[1];
				segments.push(part);
				paths.push(part);
				return {
					paths: paths,
					segments: segments
				};
			};
			let { paths } = getPaths(context);
			expect(paths).not.toContain(undefined);
			if (request === "a") {
				expect(paths).toEqual(["data:text/", "data:text/"])
				return callback(null, "42")
			}
			return callback()
		}
	]
};
