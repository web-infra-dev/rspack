// @ts-check

/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		"zod",
		"zod-validation-error",
		"json-parse-even-better-errors",
		"neo-async",
		"graceful-fs",
		{
			name: "watchpack",
			externals: {
				"graceful-fs": "../graceful-fs"
			}
		},
		{
			name: "browserslist",
			ignoreDts: true,
			externals: {
				"caniuse-lite": "caniuse-lite",
				"/^caniuse-lite(/.*)/": "caniuse-lite$1"
			}
		}
	]
};
