const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

const INCOMPLETE_STATS_WARNING =
	"Stats output may be incomplete because some compilation artifacts were unavailable";

let staleCompilation = null;
let staleInnerStats = null;
let warningChecked = false;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("PLUGIN", () => {
					if (!staleCompilation || !staleInnerStats || warningChecked) {
						return;
					}
					warningChecked = true;

					setTimeout(() => {
						const options = staleCompilation.createStatsOptions(
							{ all: true },
							{ forToString: false }
						);

						const oldWarn = console.warn;
						const warningLogs = [];
						console.warn = (...args) => {
							warningLogs.push(args.map(item => String(item)).join(" "));
						};

						const capture = captureStdio(process.stderr);
						const json = staleInnerStats.toJson(options);
						const warningOutput = capture.toString();
						capture.restore();
						console.warn = oldWarn;

						if (!json || typeof json !== "object") {
							throw new Error("Expected stats json to be an object");
						}

						const warnings = `${warningLogs.join("\n")}\n${warningOutput}`;
						if (
							!warnings.includes(INCOMPLETE_STATS_WARNING) ||
							!warnings.includes("compiler.hooks.done")
						) {
							throw new Error(
								`Expected incomplete stats warning to be printed, got: ${warnings}`
							);
						}
					});
				});

				compiler.hooks.done.tap("PLUGIN", stats => {
					if (!staleCompilation) {
						staleCompilation = stats.compilation;
						staleInnerStats = staleCompilation.__internal_getInner().getStats();
					}
				});
			}
		}
	]
};
