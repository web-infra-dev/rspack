const yargs = require("yargs");
const { hideBin } = require("yargs/helpers");
const { distDiff } = require("./dist_diff");
yargs(process.argv.slice(2))
	.version()
	.command(
		"dist <rspack_dist> <webpack_dist>",
		"diff dist of rspack and webpack output dist",
		yargs => {
			yargs
				.positional("rspack_dist", {
					type: "string",
					describe: "dist path of rspack"
				})
				.positional("webpack_dist", {
					type: "string",
					describe: "dist path of webpack"
				})
				.option("dry", {
					type: "boolean",
					default: true,
					describe: "whether use difftastic to diff the module"
				});
			return yargs;
		},
		args => {
			return distDiff(args);
		}
	)
	.command(
		"stats <rspack_stats> <webpack_stats>",
		"diff stats of rspack and webpack",
		yargs => {
			yargs
				.positional("rspack_stats", {
					type: "string",
					describe: "stats path of rspack"
				})
				.positional("webpack_stats", {
					type: "string",
					describe: "stats path of rspack"
				});
			return yargs;
		},
		args => {
			return statDiff();
		}
	)
	.showHelpOnFail()
	.parse();
