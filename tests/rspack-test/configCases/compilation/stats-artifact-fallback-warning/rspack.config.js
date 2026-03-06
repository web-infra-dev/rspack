const captureStdio = require("@rspack/test-tools/helper/legacy/captureStdio");

const INCOMPLETE_STATS_WARNING =
	"Stats output may be incomplete because some compilation artifacts were unavailable (exportsInfo). For complete stats data, call `stats.toJson()` inside `compiler.hooks.done`.";

let warningChecked = false;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("PLUGIN", compilation => {
					compilation.hooks.finishModules.tap("PLUGIN", () => {
						if (warningChecked) {
							return;
						}
						warningChecked = true;

						const oldWarn = console.warn;
						const warningLogs = [];
						console.warn = (...args) => {
							warningLogs.push(args.map(item => String(item)).join(" "));
						};

						const capture = captureStdio(process.stderr);
						const json = compilation.getStats().toJson({ all: true });
						const warningOutput = capture.toString();
						capture.restore();
						console.warn = oldWarn;

						if (!json || typeof json !== "object") {
							throw new Error("Expected stats json to be an object");
						}

						if (!warningLogs.includes(INCOMPLETE_STATS_WARNING)) {
							throw new Error(
								`Expected incomplete stats warning.
Expected: ${INCOMPLETE_STATS_WARNING}
Actual: ${warningLogs.join("\n")}\nStderr: ${warningOutput}`
							);
						}
					});
				});
			}
		}
	],
	module: {
		rules: [
			{
				test: /\.js$/,
				use: [require.resolve("./warning-loader")]
			}
		]
	}
};
