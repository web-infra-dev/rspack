module.exports = {
	mode: "development",
	devtool: false,
	optimization: { concatenateModules: false },
	plugins: [{
		apply(compiler) {
			compiler.hooks.thisCompilation.tap("StableAbsenceLog", compilation => {
				compilation.hooks.processAssets.tap({ name: "StableAbsenceLog", stage: 5000 }, () => {
					const logging = compilation.getStats().toJson({ all: false, logging: "verbose" }).logging;
					const entries = logging?.["rspack.Compilation.codeSplittingCache"]?.entries || [];
					const lines = entries.map(entry => entry.message);
					compilation.emitAsset("cache-log.json", new compiler.rspack.sources.RawSource(JSON.stringify(lines)));
					const graphLines = (logging?.["rspack.Compilation"]?.entries || []).map(entry => entry.message);
					compilation.emitAsset("graph-log.json", new compiler.rspack.sources.RawSource(JSON.stringify(graphLines)));
				});
			});
		}
	}]
};
