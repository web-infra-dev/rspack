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
	// The working directory of where napi command will be executed in, all other paths options are relative to this path
	cwd: {
		type: "string",
		description:
			"The working directory of where napi command will be executed in, all other paths options are relative to this path"
	},
	// Path to `Cargo.toml`
	"manifest-path": { type: "string", description: "Path to `Cargo.toml`" },
	// Path to `napi` config json file
	"config-path": {
		type: "string",
		short: "c",
		description: "Path to `napi` config json file"
	},
	// Path to `package.json`
	"package-json-path": {
		type: "string",
		description: "Path to `package.json`"
	},
	// Directory for all crate generated artifacts, see `cargo build --target-dir`
	"target-dir": {
		type: "string",
		description:
			"Directory for all crate generated artifacts, see `cargo build --target-dir`"
	},
	// Path to where all the built files would be put. Default to the crate folder
	"output-dir": {
		type: "string",
		short: "o",
		description:
			"Path to where all the built files would be put. Default to the crate folder"
	},

	// Platform and JS options
	// Add platform triple to the generated nodejs binding file, eg: `[name].linux-x64-gnu.node`
	platform: {
		type: "boolean",
		description:
			"Add platform triple to the generated nodejs binding file, eg: `[name].linux-x64-gnu.node`"
	},
	// Package name in generated js binding file. Only works with `--platform` flag
	"js-package-name": {
		type: "string",
		description:
			"Package name in generated js binding file. Only works with `--platform` flag"
	},
	// Whether generate const enum for typescript bindings
	// "const-enum": { type: "boolean", description: "Whether generate const enum for typescript bindings" },
	// Path and filename of generated JS binding file. Only works with `--platform` flag. Relative to `--output-dir`.
	js: {
		type: "string",
		description:
			"Path and filename of generated JS binding file. Only works with `--platform` flag. Relative to `--output-dir`."
	},
	// Whether to disable the generation JS binding file. Only works with `--platform` flag.
	"no-js": {
		type: "boolean",
		description:
			"Whether to disable the generation JS binding file. Only works with `--platform` flag."
	},

	// TypeScript definition options
	// Path and filename of generated type def file. Relative to `--output-dir`
	dts: {
		type: "string",
		description:
			"Path and filename of generated type def file. Relative to `--output-dir`"
	},
	// Custom file header for generated type def file. Only works when `typedef` feature enabled.
	"dts-header": {
		type: "string",
		description:
			"Custom file header for generated type def file. Only works when `typedef` feature enabled."
	},
	// Whether to disable the default file header for generated type def file. Only works when `typedef` feature enabled.
	"no-dts-header": {
		type: "boolean",
		description:
			"Whether to disable the default file header for generated type def file. Only works when `typedef` feature enabled."
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
	// Whether to emit an ESM JS binding file instead of CJS format. Only works with `--platform` flag.
	esm: {
		type: "boolean",
		description:
			"Whether to emit an ESM JS binding file instead of CJS format. Only works with `--platform` flag."
	},
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

	// Package and binary options
	// Build only the specified binary
	bin: { type: "string", description: "Build only the specified binary" },
	// Build the specified library or the one at cwd
	package: {
		type: "string",
		short: "p",
		description: "Build the specified library or the one at cwd"
	},

	// Cross compilation options
	// [experimental] cross-compile for the specified target with `cargo-xwin` on windows and `cargo-zigbuild` on other platform
	// "cross-compile": { type: "boolean", short: "x", description: "[experimental] cross-compile for the specified target with `cargo-xwin` on windows and `cargo-zigbuild` on other platform" },
	// [experimental] use [cross](https://github.com/cross-rs/cross) instead of `cargo`
	// "use-cross": { type: "boolean", description: "[experimental] use [cross](https://github.com/cross-rs/cross) instead of `cargo`" },
	// [experimental] use @napi-rs/cross-toolchain to cross-compile Linux arm/arm64/x64 gnu targets.
	// "use-napi-cross": { type: "boolean", description: "[experimental] use @napi-rs/cross-toolchain to cross-compile Linux arm/arm64/x64 gnu targets." },

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
		description: "Space-separated list of features to activate"
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
Usage: node scripts/build.js [options]

Build the NAPI-RS project

Options:`);

	const categories = {
		"Build target and paths": [
			"target",
			"cwd",
			"manifest-path",
			"config-path",
			"package-json-path",
			"target-dir",
			"output-dir"
		],
		"Platform and JS options": ["platform", "js-package-name", "js", "no-js"],
		"TypeScript definition options": [
			"dts",
			"dts-header",
			"no-dts-header",
			"dts-cache"
		],
		"Output format options": ["esm", "strip"],
		"Build mode options": ["release", "verbose", "profile"],
		"Package and binary options": ["bin", "package"],
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
		if (values.cwd) buildOptions.cwd = values.cwd;
		if (values["manifest-path"])
			buildOptions.manifestPath = values["manifest-path"];
		if (values["config-path"]) buildOptions.configPath = values["config-path"];
		if (values["package-json-path"])
			buildOptions.packageJsonPath = values["package-json-path"];
		if (values["target-dir"]) buildOptions.targetDir = values["target-dir"];
		if (values["output-dir"]) buildOptions.outputDir = values["output-dir"];

		// Platform and JS options
		if (values.platform) buildOptions.platform = values.platform;
		if (values["js-package-name"])
			buildOptions.jsPackageName = values["js-package-name"];
		// if (values["const-enum"]) buildOptions.constEnum = values["const-enum"];
		if (values.js) buildOptions.jsBinding = values.js;
		if (values["no-js"]) buildOptions.noJsBinding = values["no-js"];

		// TypeScript definition options
		if (values.dts) buildOptions.dts = values.dts;
		if (values["dts-header"]) buildOptions.dtsHeader = values["dts-header"];
		if (values["no-dts-header"])
			buildOptions.noDtsHeader = values["no-dts-header"];
		if (values["dts-cache"] !== undefined)
			buildOptions.dtsCache = values["dts-cache"];
		if (values["no-dts-cache"]) buildOptions.dtsCache = false;

		// Output format options
		if (values.esm) buildOptions.esm = values.esm;
		if (values.strip) buildOptions.strip = values.strip;

		// Build mode options
		if (values.release) buildOptions.release = values.release;
		if (values.verbose) buildOptions.verbose = values.verbose;
		if (values.profile) buildOptions.profile = values.profile;

		// Package and binary options
		if (values.bin) buildOptions.bin = values.bin;
		if (values.package) buildOptions.package = values.package;

		// Cross compilation options
		// if (values["cross-compile"]) buildOptions.crossCompile = values["cross-compile"];
		// if (values["use-cross"]) buildOptions.useCross = values["use-cross"];
		// if (values["use-napi-cross"]) buildOptions.useNapiCross = values["use-napi-cross"];

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
		const dts = path.resolve(buildOptions.cwd, "binding.d.ts");
		let dtsContent = readFileSync(dts, "utf8");

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
