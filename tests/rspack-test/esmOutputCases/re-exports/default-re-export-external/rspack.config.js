module.exports = {
	externals: {
		fs: "module fs"
	},
	optimization: {
		concatenateModules: true,
		usedExports: true
	}
};
