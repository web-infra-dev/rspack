const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

const INCOMPLETE_STATS_WARNING_PREFIX =
	"Stats output may be incomplete because some compilation artifacts were unavailable (";
const INCOMPLETE_STATS_WARNING_SUFFIX =
	"). For complete stats data, call `stats.toJson()` inside `compiler.hooks.done`.";
const ALLOWED_FALLBACK_ARTIFACTS = new Set([
	"buildModuleGraph",
	"exportsInfo",
	"moduleGraph",
	"moduleIds",
	"chunkHashes"
]);

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

						if (warningLogs.length !== 1) {
							throw new Error(
								`Expected exactly one incomplete stats warning.\nActual: ${warningLogs.join(
									"\n"
								)}\nStderr: ${warningOutput}`
							);
						}

						const warning = warningLogs[0];
						if (
							!warning.startsWith(INCOMPLETE_STATS_WARNING_PREFIX) ||
							!warning.endsWith(INCOMPLETE_STATS_WARNING_SUFFIX)
						) {
							throw new Error(
								`Expected incomplete stats warning format.\nActual: ${warning}\nStderr: ${warningOutput}`
							);
						}

						const artifactList = warning
							.slice(
								INCOMPLETE_STATS_WARNING_PREFIX.length,
								warning.length - INCOMPLETE_STATS_WARNING_SUFFIX.length
							)
							.split(",")
							.map(item => item.trim())
							.filter(Boolean);

						if (
							artifactList.length === 0 ||
							artifactList.some(item => !ALLOWED_FALLBACK_ARTIFACTS.has(item))
						) {
							throw new Error(
								`Expected known fallback artifact names.\nActual warning: ${warning}\nParsed artifacts: ${artifactList.join(
									", "
								)}\nStderr: ${warningOutput}`
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
