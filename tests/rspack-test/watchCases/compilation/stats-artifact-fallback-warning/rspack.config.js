const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

const INCOMPLETE_STATS_WARNING =
	"Stats output may be incomplete because some compilation artifacts were unavailable (buildModuleGraph). For complete stats data, call `stats.toJson()` inside `compiler.hooks.done`.";

let staleStats = null;
let warningChecked = false;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("PLUGIN", () => {
					if (!staleStats || warningChecked) {
						return;
					}
					warningChecked = true;

					setTimeout(() => {
						const options = staleStats.compilation.createStatsOptions(
							{ all: true },
							{ forToString: false }
						);

						const oldWarn = console.warn;
						const warningLogs = [];
						console.warn = (...args) => {
							warningLogs.push(args.map(item => String(item)).join(" "));
						};

						const capture = captureStdio(process.stderr);
						const json = staleStats.toJson(options);
						const warningOutput = capture.toString();
						capture.restore();
						console.warn = oldWarn;

						if (!json || typeof json !== "object") {
							throw new Error("Expected stats json to be an object");
						}

						if (warningLogs.length !== 1 || warningLogs[0] !== INCOMPLETE_STATS_WARNING) {
							throw new Error(
								`Expected exact incomplete stats warning.\nExpected: ${INCOMPLETE_STATS_WARNING}\nActual: ${warningLogs.join(
									"\n"
								)}\nStderr: ${warningOutput}`
							);
						}
					});
				});

				compiler.hooks.done.tap("PLUGIN", stats => {
					if (!staleStats) {
						staleStats = stats;
					}
				});
			}
		}
	]
};
