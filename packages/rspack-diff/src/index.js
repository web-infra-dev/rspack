const yargs = require("yargs");
const { hideBin } = require("yargs/helpers");
const { distDiffBundler } = require("./dist_diff");
yargs(process.argv.slice(2))
	.version()
	.command(
		"dist <src> <dst>",
		"diff dist of rspack and webpack output dist",
		yargs => {
			yargs
				.positional("src", {
					type: "string",
					describe: "dist path of src"
				})
				.positional("dst", {
					type: "string",
					describe: "dist path of dst"
				});
			return yargs;
		},
		arg => {
			return distDiffBundler(arg.src, arg.dst);
		}
	)
	.showHelpOnFail()
	.parse();
