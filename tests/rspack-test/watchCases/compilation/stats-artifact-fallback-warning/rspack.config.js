const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

// Verify stale Stats object captured from done hook follows JS-side fallback path
// and does not trigger native artifact-based incomplete warnings.
// This aligns with watch incremental behavior: stale `stats` may outlive internal
// graph rebuild, so JS wrapper should short-circuit with placeholder output.
const STALE_STATS_HASH = "XXXX";

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

						if (warningLogs.length !== 0) {
							throw new Error(
								`Expected stale stats to use JS-side fallback (no warning), got: ${warningLogs.join(
									"\n"
								)}\nStderr: ${warningOutput}`
							);
						}

						if (json.hash !== STALE_STATS_HASH) {
							throw new Error(
								`Expected stale stats placeholder hash ${STALE_STATS_HASH}, got ${json.hash}`
							);
						}
					});
				});

				compiler.hooks.done.tap("PLUGIN", stats => {
					if (!staleStats) {
						staleStats = stats;
					}
				});
			},
		},
	],
};
