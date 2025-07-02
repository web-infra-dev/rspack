// @ts-check
/** @type {import('prebundle').Config} */
export default {
	dependencies: [
		{
			name: "webpack-sources",
			copyDts: true
		},
		"graceful-fs",
		"browserslist-load-config",
		{
			name: "watchpack",
			externals: {
				"graceful-fs": "../graceful-fs/index.js"
			}
		},
		{
			name: "@swc/types"
		}
	]
};
