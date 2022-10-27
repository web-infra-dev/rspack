import yargs from "yargs";
export const commonOptions = (yargs: yargs.Argv<{}>) => {
	return yargs
		.positional("entry", {
			type: "string",
			array: true,
			describe: "entry"
		})
		.options({
			config: {
				g: true,
				type: "string",
				describe: "config file",
				alias: "c"
			},
			mode: { type: "string", describe: "mode" },
			watch: {
				type: "boolean",
				default: false,
				describe: "watch"
			},
			devtool: {
				type: "boolean",
				default: false,
				describe: "devtool"
			}
		});
};
