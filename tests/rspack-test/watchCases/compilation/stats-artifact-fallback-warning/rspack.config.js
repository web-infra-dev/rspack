const path = require("path");
const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

const INCOMPLETE_STATS_WARNING =
	"Stats output may be incomplete because some compilation artifacts were unavailable";

let staleCompilation = null;
let staleInnerStats = null;
let warningChecked = false;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /index\\.js$/,
				use: [path.join(__dirname, "warning-loader.js")]
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("PLUGIN", () => {
					if (!staleCompilation || !staleInnerStats || warningChecked) {
						return;
					}
					warningChecked = true;

					setTimeout(() => {
						const statsOptions = staleCompilation.createStatsOptions(
							{ all: false, errors: true, warnings: true },
							{ forToString: false }
						);

						const oldWarn = console.warn;
						const warningLogs = [];
						console.warn = (...args) => {
							warningLogs.push(args.map(item => String(item)).join(" "));
						};

						const capture = captureStdio(process.stderr);
						const json = staleInnerStats.toJson(statsOptions);
						const warningOutput = capture.toString();
						capture.restore();
						console.warn = oldWarn;

						if (!json || typeof json !== "object") {
							throw new Error("Expected binding stats json to be an object");
						}

						const fallbackFlags =
							typeof staleInnerStats.__internal_getArtifactFallbackFlags ===
							"function"
								? staleInnerStats.__internal_getArtifactFallbackFlags()
								: "unknown";
						const warningMessages = warningLogs.join("\n") + "\n" + warningOutput;

						if (
							!warningMessages.includes(INCOMPLETE_STATS_WARNING) ||
							!warningMessages.includes("compiler.hooks.done")
						) {
							throw new Error(
								`Expected incomplete stats warning to be printed, flags=${fallbackFlags}, got: ${warningMessages}`
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
