const path = require("node:path");
const { readFileSync, writeFileSync } = require("node:fs");

const debug = require("debug")("rspack-builder:build");

const OPTIONS_CONFIG = {
	// Build target and paths
	// Build for the target triple, bypassed to `cargo build --target`
	target: {
		type: "string",
		short: "t",
		description:
			"Build for the target triple, bypassed to `cargo build --target`"
	},
	// Path to `package.json`
	"package-json-path": {
		type: "string",
		description: "Path to `package.json`"
	},
	// Platform and JS options
	// TypeScript definition options
	// Path and filename of generated type def file. Relative to `--output-dir`
	dts: {
		type: "string",
		description:
			"Path and filename of generated type def file. Relative to `--output-dir`"
	},
	// Whether to enable the dts cache, default to true
	"dts-cache": {
		type: "boolean",
		description: "Whether to enable the dts cache, default to enable dts cache"
	},
	"no-dts-cache": {
		type: "boolean",
		description: "Whether to enable the dts cache, default to enable dts cache"
	},

	// Output format options
	// Whether strip the library to achieve the minimum file size
	strip: {
		type: "boolean",
		short: "s",
		description: "Whether strip the library to achieve the minimum file size"
	},

	// Build mode options
	// Build in release mode
	release: {
		type: "boolean",
		short: "r",
		description: "Build in release mode"
	},
	// Verbosely log build command trace
	verbose: {
		type: "boolean",
		short: "v",
		description: "Verbosely log build command trace"
	},
	// Build artifacts with the specified profile
	profile: {
		type: "string",
		description: "Build artifacts with the specified profile"
	},

	// Watch mode
	// watch the crate changes and build continuously with `cargo-watch` crates
	watch: {
		type: "boolean",
		short: "w",
		description:
			"watch the crate changes and build continuously with `cargo-watch` crates"
	},

	// Feature options
	// Comma-separated list of features to activate
	features: {
		type: "string",
		multiple: true,
		description: "Comma-separated list of features to activate"
	},
	// Activate all available features
	"all-features": {
		type: "boolean",
		description: "Activate all available features"
	},
	// Do not activate the `default` feature
	"no-default-features": {
		type: "boolean",
		description: "Do not activate the `default` feature"
	},

	// Help option
	help: { type: "boolean", short: "h", description: "Show help information" }
};

function showHelp() {
	console.log(`
Usage: rspack-builder [options]

Build the Rspack binding builder crate

Options:`);

	const categories = {
		"Build target and paths": [
			"target",
			"package-json-path"
		],
		"TypeScript definition options": [
			"dts",
			"dts-cache",
			"no-dts-cache"
		],
		"Output format options": ["strip"],
		"Build mode options": ["release", "verbose", "profile"],
		"Watch mode": ["watch"],
		"Feature options": ["features", "all-features", "no-default-features"],
		Help: ["help"]
	};

	for (const [category, options] of Object.entries(categories)) {
		console.log(`\n  ${category}:`);
		for (const optionName of options) {
			const config = OPTIONS_CONFIG[optionName];
			if (!config) continue;

			const short = config.short ? `-${config.short}, ` : "    ";
			const long = `--${optionName}`;
			const type =
				config.type === "boolean"
					? ""
					: config.multiple
						? " <values...>"
						: " <value>";
			const description = config.description || "";

			console.log(
				`    ${short}${long}${type.padEnd(20 - long.length)} ${description}`
			);
		}
	}
	console.log(`
Examples:
  rspack-builder                           # Basic build
  rspack-builder --release                 # Release build
  rspack-builder --target x86_64-unknown-linux-musl
`);
}

const parseArgsOptions = Object.fromEntries(
	Object.entries(OPTIONS_CONFIG).map(([key, config]) => [
		key,
		{
			type: config.type,
			...(config.short && { short: config.short }),
			...(config.multiple && { multiple: config.multiple })
		}
	])
);

const { values, positionals } = require("node:util").parseArgs({
	args: process.argv.slice(2),
	options: parseArgsOptions,
	strict: true,
	allowPositionals: true
});

if (values.help) {
	showHelp();
	process.exit(0);
}

const { NapiCli } = require("@napi-rs/cli");

const CARGO_SAFELY_EXIT_CODE = 0;

debug("rspack binding builder cli received arguments:", values);
debug("rspack binding builder cli received positionals:", positionals);

build()
	.then(value => {
		// Regarding cargo's non-zero exit code as an error.
		if (value !== CARGO_SAFELY_EXIT_CODE) {
			process.exit(value);
		}
	})
	.catch(err => {
		console.error(err);
		process.exit(1);
	});

async function build() {
	try {
		const CWD = process.cwd();
		const features = [];
		/**
		 * @type {Parameters<InstanceType<typeof import('@napi-rs/cli').NapiCli>['build']>[0]}
		 */
		const buildOptions = {};

		// Map all parsed arguments to buildOptions
		if (values.target) buildOptions.target = values.target;
		if (values["package-json-path"])
			buildOptions.packageJsonPath = values["package-json-path"];

		// TypeScript definition options
		if (values.dts) buildOptions.dts = values.dts;
		if (values["dts-cache"] !== undefined)
			buildOptions.dtsCache = values["dts-cache"];
		if (values["no-dts-cache"]) buildOptions.dtsCache = false;

		// Output format options
		if (values.strip) buildOptions.strip = values.strip;

		// Build mode options
		if (values.release) buildOptions.release = values.release;
		if (values.verbose) buildOptions.verbose = values.verbose;
		if (values.profile) buildOptions.profile = values.profile;

		// Watch mode
		if (values.watch) buildOptions.watch = values.watch;

		// Feature options
		if (values.features) features.push(...values.features);
		if (values["all-features"])
			buildOptions.allFeatures = values["all-features"];
		if (values["no-default-features"])
			buildOptions.noDefaultFeatures = values["no-default-features"];

		// Set default values if not provided
		if (!buildOptions.cwd) buildOptions.cwd = CWD;
		if (!buildOptions.platform) buildOptions.platform = true;

		// Set default values
		buildOptions.noJsBinding = true;
		buildOptions.noDtsHeader = true;

		if (features.length) {
			// NAPI-RS CLI handles features as a space-separated list, this is not accepted by cargo.
			// We need to convert it to a comma-separated list.
			buildOptions.features = [features.join(",")];
		}

		// Handle positional arguments (cargo options)
		if (positionals.length > 0) {
			buildOptions.cargoOptions = positionals;
		}

		const cli = new NapiCli();
		const { task } = await cli.build(buildOptions);
		const outputs = await task;

		console.log("Build completed successfully");

		// Fix an issue where napi cli does not generate `string_enum` with `enum`s.
		const dts = path.resolve(buildOptions.cwd, buildOptions.dts);
		let dtsContent = readFileSync(dts, "utf8");

		const dtsLines = dtsContent.split("\n");
		let endHeaderIndex
		// clean old dts header
		if ((endHeaderIndex = dtsLines.findIndex(line => line.startsWith("/* -- napi-rs generated below -- */"))) && endHeaderIndex !== -1) {
			dtsContent = dtsLines.slice(endHeaderIndex + 1).join("\n");
		}

		const headers = [readFileSync(path.resolve(__dirname, "banner.d.ts"))];
		dtsContent = prependDtsHeaderFile(dtsContent, headers);

		writeFileSync(
			dts,
			dtsContent
				.replaceAll("const enum", "enum")
				// Remove the NormalModule type declaration generated by N-API.
				// We manually declare the NormalModule type in banner.d.ts
				// This allows users to extend NormalModule with static methods through type augmentation.
				.replaceAll(
					/export\s+declare\s+class\s+NormalModule\s*\{([\s\S]*?)\}\s*(?=\n\s*(?:export|declare|class|$))/g,
					""
				)
		);

		return CARGO_SAFELY_EXIT_CODE;
	} catch (error) {
		console.error("Build failed:", error);
		return 1;
	}
}

function removeBOM(dts) {
	if (dts[0] === 0xfeff) {
		return dts.slice(1);
	}
	return dts;
}

function prependDtsHeaderFile(dts, headers) {
	const banner = headers.map(removeBOM).join("\n");
	return [banner, dts].join("\n");
}
