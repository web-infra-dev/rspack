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
			env: {
				type: "array",
				string: true,
				describe: "env passed to config function"
			},
			"node-env": {
				string: true,
				describe: "sets process.env.NODE_ENV to be specified value"
			},
			devtool: {
				type: "boolean",
				default: false,
				describe: "devtool"
			},
			configName: {
				type: "array",
				string: true,
				describe: "Name of the configuration to use."
			}
		});
};

export const previewOptions = (yargs: yargs.Argv<{}>) => {
	return yargs
		.positional("root", {
			type: "string",
			describe: "root"
		})
		.options({
			publicPath: {
				type: "string",
				describe: "public path"
			},
			config: {
				g: true,
				type: "string",
				describe: "config file",
				alias: "c"
			},
			port: {
				type: "number",
				describe: "port"
			},
			host: {
				type: "string",
				describe: "host"
			},
			open: {
				type: "boolean",
				describe: "open browser"
			},
			// same as devServer.server
			server: {
				type: "string",
				describe: "Configuration items for the server."
			},
			configName: {
				type: "array",
				string: true,
				describe: "Name of the configuration to use."
			}
		});
};

export function normalizeEnv(argv) {
	function parseValue(previous, value) {
		const [allKeys, val] = value.split(/=(.+)/, 2);
		const splitKeys = allKeys.split(/\.(?!$)/);

		let prevRef = previous;

		splitKeys.forEach((someKey, index) => {
			// https://github.com/webpack/webpack-cli/issues/3284
			if (someKey.endsWith("=")) {
				// remove '=' from key
				someKey = someKey.slice(0, -1);
				prevRef[someKey] = undefined;
				return;
			}

			if (!prevRef[someKey]) {
				prevRef[someKey] = {};
			}

			if (typeof prevRef[someKey] === "string") {
				prevRef[someKey] = {};
			}

			if (index === splitKeys.length - 1) {
				if (typeof val === "string") {
					prevRef[someKey] = val;
				} else {
					prevRef[someKey] = true;
				}
			}

			prevRef = prevRef[someKey] as Record<string, string | object | boolean>;
		});

		return previous;
	}
	const envObj = ((argv.env as string[]) ?? []).reduce(parseValue, {});
	argv.env = envObj;
}
