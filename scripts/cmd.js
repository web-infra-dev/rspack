const cp = require("child_process");
const log = require("./log");
const { Command, command } = require("commander");

const COMMANDER_VERSION = "2.20.3";

function checkCommandVersion() {
	const pkgInfo = require(require.resolve("commander/package.json"));
	if (pkgInfo.version !== COMMANDER_VERSION) {
		log.error(
			`expected the version of Commander is 2.20.3, yours is ${pkgInfo.version}`
		);
		process.exit(1);
	}
}

function createCLI() {
	checkCommandVersion();

	const cli = new Command();

	cli.description("cli to control project").version("0.1.0");

	cli
		.command("install")
		.alias("i")
		.description("install node dependencies")
		.action(() => {
			const COMMAND = "pnpm i";
			log.info(`start install deps by '${COMMAND}'`);
			cp.execSync(COMMAND, {
				stdio: "inherit"
			});
			log.info("finish install deps");
		});
	cli
		.command("test")
		.option("rust", "run all rust test")
		.option("example", "test arco pro in rust side")
		.option("js", "test all js packages")
		.action(args => {
			let command;
			switch (args) {
				case "rust":
					command = "cargo test --all -- --nocapture";
					break;
				case "js":
					command = "pnpm -r run test";
					break;
				case "example":
					command = "cargo run --example arco_pro";
					break;
				default:
					log.error("invalid args, see `./x test -h` to get more information");
					process.exit(1);
			}
			if (!command) {
				return;
			}
			log.info(`start test by '${command}'`);
			cp.execSync(command, {
				stdio: "inherit"
			});
			log.info("test finished");
		});
	cli
		.command("dev")
		.description("run dev for ")
		.option("js", "dev for all js package")
		.action(args => {
			let command;
			switch (args) {
				case "js":
					command = `pnpm -parallel --filter "@rspack/*" dev`;
					break;
				default:
					log.error(`invalid args, see "./x dev -h" to get more information`);
					process.exit(1);
			}
			cp.execSync(command, {
				stdio: "inherit"
			});
		});
	cli
		.command("build")
		.option("binding", "build binding between rust and js")
		.option("cli", "build @rspack/core which located in js side")
		.option("bundle", "build example directory in rust side")
		.option("webpack", "build webpack-example directory")
		.option("js", "build all js library")
		.option("examples", "build all rspack examples")
		.action(args => {
			let command;
			switch (args) {
				case "js":
					command = `pnpm --filter "@rspack/*" build`;
					break;
				case "binding":
					command = "pnpm --filter @rspack/binding build:debug";
					break;
				case "cli":
					command = `pnpm --filter @rspack/binding build:debug && pnpm --filter "@rspack/*" build`;
					break;
				case "cli:release": // only build local release binary, for benchmark
					command = `pnpm --filter @rspack/binding build:release && pnpm --filter "@rspack/*" build`;
					break;
				case "cli:release:all": // build for all cross platform, for release
					command = `pnpm --filter @rspack/binding build:release:all && pnpm --filter "@rspack/*" build`;
					break;
				case "cli:debug": // only build local debug release, for local debug
					command = `pnpm --filter @rspack/binding build:debug && pnpm --filter "@rspack/*" build`;
					break;
				case "bundle":
					command = "cargo run --package rspack --example bundle";
					break;
				case "examples":
					command = 'pnpm --filter "example-*" build';
					break;
				default:
					log.error("invalid args, see `./x build -h` to get more information");
					process.exit(1);
			}
			if (!command) {
				return;
			}
			log.info(`start build by '${command}'`);
			cp.execSync(command, {
				stdio: "inherit"
			});
			log.info("build finished");
		});

	cli
		.command("format")
		.option("rs", "format rust code")
		.option("js", "format js code")
		.option("toml", "format toml code")
		.action(args => {
			let command;
			switch (args) {
				case "js":
					command =
						'npx prettier "packages/**/*.{ts,js}" "crates/rspack_plugin_runtime/**/*.{ts,js}" --check --write';
					break;
				case "rs":
					command = "pnpm --filter @rspack/core... build";
					break;
				case "toml":
					command =
						"npx @taplo/cli format --check '.cargo/*.toml' './crates/**/Cargo.toml' './Cargo.toml'";
					break;
				default:
					log.error(
						"invalid args, see `./x format -h` to get more information"
					);
					process.exit(1);
			}
			if (!command) {
				return;
			}
			log.info(`start format by '${command}'`);
			cp.execSync(command, {
				stdio: "inherit"
			});
			log.info("format finished");
		});

	cli
		.command("lint")
		.option("js", "lint js code")
		.option("rs", "lint rust code")
		.action(args => {
			let commands = [];
			switch (args) {
				case "js":
					commands = ['npx prettier "packages/**/*.{ts,js}" --check'];
					break;
				case "rs":
					commands = [
						"cargo clippy --all -- --deny warnings",
						"node ./scripts/check_rust_dependency.js"
					];
					break;
				default:
					log.error(
						"invalid args, see `./x format -h` to get more information"
					);
					process.exit(1);
			}
			commands.forEach(command => {
				cp.execSync(command, {
					stdio: "inherit"
				});
			});
			log.info("lint finished");
		});
	cli
		.command("clean")
		.option("all", "clean all")
		.option("dist", "clean build artifacts")
		.option("npm", "clean node_modules")
		.action(args => {
			let commands = [];
			let clean_npm = `rimraf node_modules && rimraf packages/**/node_modules`;
			let clean_rust = `cargo clean`;
			let clean_dist = `rimraf packages/**/{lib,dist}`;
			log.info("start clean");
			switch (args) {
				case "all":
					commands = [clean_npm, clean_rust, clean_dist];
					break;
				case "npm":
					commands = [clean_npm];
					break;
				case "dist":
					commands = [clean_dist];
			}
			commands.forEach(command => {
				cp.execSync(command, {
					stdio: "inherit"
				});
			});
			log.info("finish clean");
		});

	cli
		.command("script")
		.option(
			"update_swc_version",
			"update all swc sub packages to the correct version"
		)
		.action(args => {
			let commands = [];
			switch (args) {
				case "update_swc_version":
					commands = ["node ./scripts/update_swc_version.js"];
					break;
				default:
					log.error(
						"invalid args, see `./x format -h` to get more information"
					);
					process.exit(1);
			}
			commands.forEach(command => {
				cp.execSync(command, {
					stdio: "inherit"
				});
			});
			log.info("run script finished");
		});
	cli
		.command("pkg-version")
		.option("show package version for release note")
		.action(args => {
			const { version } = require("../packages/rspack-cli/package.json");
			console.log(version);
		});
	return cli;
}

module.exports = { createCLI };
