import type yargs from "yargs";

/**
 * Apply common options for all commands
 */
export const commonOptions = (yargs: yargs.Argv) => {
	return yargs.options({
		config: {
			g: true,
			type: "string",
			describe: "config file",
			alias: "c"
		},
		configName: {
			type: "array",
			string: true,
			describe: "Name of the configuration to use."
		},
		configLoader: {
			type: "string",
			default: "register",
			describe:
				"Specify the loader to load the config file, can be `native` or `register`."
		},
		nodeEnv: {
			string: true,
			describe: "sets `process.env.NODE_ENV` to be specified value"
		}
	});
};

/**
 * Apply common options for `build` and `serve` commands
 */
export const commonOptionsForBuildAndServe = (yargs: yargs.Argv) => {
	return yargs
		.options({
			entry: {
				type: "array",
				string: true,
				describe: "entry file"
			},
			outputPath: {
				type: "string",
				describe: "output path dir",
				alias: "o"
			},
			mode: { type: "string", describe: "mode", alias: "m" },
			watch: {
				type: "boolean",
				default: false,
				describe: "watch",
				alias: "w"
			},
			env: {
				type: "array",
				string: true,
				describe: "env passed to config function"
			},
			devtool: {
				type: "boolean",
				default: false,
				describe: "devtool",
				alias: "d"
			}
		})
		.alias({ v: "version", h: "help" });
};

export function normalizeEnv(argv: yargs.Arguments) {
	function parseValue(previous: Record<string, unknown>, value: string) {
		const [allKeys, val] = value.split(/=(.+)/, 2);
		const splitKeys = allKeys.split(/\.(?!$)/);

		let prevRef = previous;

		splitKeys.forEach((key, index) => {
			let someKey = key;

			// https://github.com/webpack/webpack-cli/issues/3284
			if (someKey.endsWith("=")) {
				// remove '=' from key
				someKey = someKey.slice(0, -1);
				prevRef[someKey] = undefined;
				return;
			}

			if (!prevRef[someKey] || typeof prevRef[someKey] === "string") {
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

/**
 * set builtin env from cli - like `WEBPACK_BUNDLE=true`. also for `RSPACK_` prefixed.
 * @param env the `argv.env` object
 * @param envNameSuffix the added env will be `WEBPACK_${envNameSuffix}` and `RSPACK_${envNameSuffix}`
 * @param value
 */
export function setBuiltinEnvArg(
	env: Record<string, any>,
	envNameSuffix: string,
	value: any
) {
	const envNames = [
		// TODO: breaking change
		// `WEBPACK_${envNameSuffix}`,
		`RSPACK_${envNameSuffix}`
	];
	for (const envName of envNames) {
		if (envName in env) {
			continue;
		}
		env[envName] = value;
	}
}

/**
 * infer `argv.env` as an object for it was transformed from array to object after `normalizeEnv` middleware
 * @returns the reference of `argv.env` object
 */
export function ensureEnvObject<T extends Record<string, unknown>>(
	options: yargs.Arguments
): T {
	if (Array.isArray(options.env)) {
		// in case that cli haven't got `normalizeEnv` middleware applied
		normalizeEnv(options);
	}
	options.env = options.env || {};
	return options.env as T;
}

export function setDefaultNodeEnv(
	options: yargs.Arguments,
	defaultEnv: string
) {
	if (process.env.NODE_ENV !== undefined) {
		return;
	}

	process.env.NODE_ENV =
		typeof options.nodeEnv === "string" ? options.nodeEnv : defaultEnv;
}
