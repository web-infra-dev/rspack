const path = require("node:path");
const { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync, renameSync, rmSync } = require("node:fs");
const { values, positionals } = require("node:util").parseArgs({
	args: process.argv.slice(2),
	options: {
		profile: {
			type: "string"
		},
		"profile-generate-dir": {
			type: "string"
		},
		"profile-output": {
			type: "string"
		},
		"profile-use": {
			type: "string"
		},
	},
	strict: true,
	allowPositionals: true
});

const { spawn, spawnSync } = require("node:child_process");

const NAPI_BINDING_DTS = "napi-binding.d.ts"
const CARGO_SAFELY_EXIT_CODE = 0;

const watch = process.argv.includes("--watch");

build().then((value) => {
	// Regarding cargo's non-zero exit code as an error.
	if (value !== CARGO_SAFELY_EXIT_CODE) {
		process.exit(value);
	}
}).catch(err => {
	console.error(err);
	process.exit(1);
});

async function build() {
	return new Promise((resolve, reject) => {
		const args = [
			"build",
			"--platform",
			"--dts",
			NAPI_BINDING_DTS,
			"--no-js",
			// "--no-const-enum",
			"--no-dts-header",
			"--pipe",
			`"node ${path.resolve(__dirname, "dts-header.js")}"`
		];
		const rustflags = []
		const features = [];
		const envs = { ...process.env };
		const profileGenerateDir = values["profile-generate-dir"]
			? path.resolve(values["profile-generate-dir"])
			: undefined;
		const profileOutput = path.resolve(values["profile-output"] || path.join(__dirname, "..", "pgo", "rspack.profdata"));
		const profileUseValue = values["profile-use"] || process.env.RSPACK_PGO_PROFILE;
		const profileUse = profileUseValue
			? path.resolve(profileUseValue)
			: undefined;
		const use_build_std = values.profile === "release"
			|| values.profile === "release-debug"
			|| values.profile === "release-wasi"
			|| values.profile === "profiling";

		if (profileGenerateDir && profileUse) {
			reject(new Error("--profile-generate-dir and --profile-use cannot be used together"));
			return;
		}

		if (values.profile) {
			args.push("--profile", values.profile);
		}
		if (watch) {
			args.push("--watch");
		}
		if (process.env.USE_NAPI_CROSS) {
			args.push("--use-napi-cross");
		}
		if (process.env.USE_ZIG) {
			args.push("--cross-compile");
		}
		if (process.env.RUST_TARGET) {
			args.push("--target", process.env.RUST_TARGET);
		}
		if (!process.env.DISABLE_PLUGIN) {
			args.push("--no-default-features");
			features.push("plugin");
		}
		if (process.env.RSPACK_TARGET_BROWSER) {
			features.push("browser")
		}
		if (values.profile !== "release") {
			features.push("perfetto");
		}
		args.push("--no-dts-cache");
		if (process.env.SFTRACE) {
			features.push("sftrace-setup");
			rustflags.push("-Zinstrument-xray=always");
		}
		if (process.env.ALLOCATIVE) {
			features.push("allocative");
			rustflags.push("--cfg=allocative");
		}
		if (process.env.TRACY) {
			features.push("tracy-client");
		}
		if (profileGenerateDir) {
			rmSync(profileGenerateDir, { recursive: true, force: true });
			mkdirSync(profileGenerateDir, { recursive: true });
			rustflags.push(`-Cprofile-generate=${profileGenerateDir}`);
			rustflags.push("--cfg=rspack_pgo_generate");
			envs.LLVM_PROFILE_FILE = path.join(profileGenerateDir, "rspack-%m-%p.profraw");
			envs.RSPACK_PGO_PROFILE_DUMP = "1";
			envs.RSPACK_BINDING = path.resolve(__dirname, "..");
		}
		if (profileUse) {
			if (!existsSync(profileUse)) {
				reject(new Error(`PGO profile not found: ${profileUse}`));
				return;
			}
			rustflags.push(`-Cprofile-use=${profileUse}`);
			// Let LLVM use the profile to apply size-oriented heuristics only to cold code.
			rustflags.push("-Cllvm-args=-pgso");
			rustflags.push("-Cllvm-args=-pgso-cold-code-only-for-instr-pgo");
		}
		if (values.profile === "release") {
			features.push("info-level");
			if (process.env.RUST_TARGET && !process.env.RUST_TARGET.includes("windows-msvc")) {
				rustflags.push("-Cforce-unwind-tables=no");
			}
		} else {
			// enable unwind-table for backtrace for non-release profile
			if (!process.env.RUST_TARGET || (process.env.RUST_TARGET && !process.env.RUST_TARGET.includes("windows-msvc"))) {
				rustflags.push("-Cforce-unwind-tables=yes");
			}

		}
		if (features.length) {
			args.push(`--features ${features.join(",")}`);
		}

		if (positionals.length > 0 || rustflags.length > 0 || use_build_std) {
			// napi need `--` to separate options and positional arguments.
			args.push("--");

			if (rustflags.length > 0) {
				const flag = rustflags.map(f => `\\"${f}\\"`).join(",");
				args.push("--config");
				args.push(`"target.'cfg(all())'.rustflags = [${flag}]"`)
			}

			if (use_build_std) {
				// allows to optimize std with current compile arguments
				// and avoids std code generate unwind table to save size.
				args.push("-Zbuild-std=panic_abort,std");
			}

			if (positionals.length > 0) {
				args.push(...positionals);
			}
		}

		console.log(`Run command: napi ${args.join(" ")}`);

		const cp = spawn("napi", args, {
			stdio: "inherit",
			shell: true,
			env: envs,
		});

		cp.on("error", reject);
		cp.on("exit", (code) => {
			if (code === CARGO_SAFELY_EXIT_CODE) {
				// Fix an issue where napi cli does not generate `string_enum` with `enum`s.
				const dts = path.resolve(__dirname, "..", NAPI_BINDING_DTS);
				writeFileSync(dts,
					readFileSync(dts, "utf8")
						.replaceAll("const enum", "enum")
						// Remove the NormalModule type declaration generated by N-API.
						// We manually declare the NormalModule type in banner.d.ts
						// This allows users to extend NormalModule with static methods through type augmentation.
						.replaceAll(/export\s+declare\s+class\s+NormalModule\s*\{([\s\S]*?)\}\s*(?=\n\s*(?:export|declare|class|$))/g, "")
				);

				// For browser wasm, we rename the artifacts to distinguish them from node wasm
				if (process.env.RSPACK_TARGET_BROWSER) {
					renameSync("rspack.wasm32-wasi.debug.wasm", "rspack.browser.debug.wasm")
					renameSync("rspack.wasm32-wasi.wasm", "rspack.browser.wasm")
				}

				if (process.env.TRACY) {
					// split debug symbols for tracy
					spawnSync('dsymutil', [
						path.resolve(__dirname, "..", "rspack.darwin-arm64.node")
					], {
						stdio: "inherit",
						shell: true,
					})
				}

				if (profileGenerateDir) {
					const buildJsEnvs = { ...envs };
					delete buildJsEnvs.RSPACK_BINDING;
					delete buildJsEnvs.RSPACK_PGO_PROFILE_DUMP;
					delete buildJsEnvs.LLVM_PROFILE_FILE;
					const buildJsResult = spawnSync("pnpm", ["run", "build:js"], {
						stdio: "inherit",
						shell: true,
						env: buildJsEnvs,
						cwd: path.resolve(__dirname, "..", "..", "..")
					});
					if (buildJsResult.status !== CARGO_SAFELY_EXIT_CODE) {
						resolve(buildJsResult.status || 1);
						return;
					}

					const benchResult = spawnSync("pnpm", ["--filter", "bench", "run", "bench"], {
						stdio: "inherit",
						shell: true,
						env: envs,
						cwd: path.resolve(__dirname, "..", "..", "..")
					});
					if (benchResult.status !== CARGO_SAFELY_EXIT_CODE) {
						resolve(benchResult.status || 1);
						return;
					}

					const profrawFiles = readdirSync(profileGenerateDir)
						.filter(file => file.endsWith(".profraw"))
						.map(file => path.join(profileGenerateDir, file));
					if (profrawFiles.length === 0) {
						reject(new Error(`No .profraw files were generated in ${profileGenerateDir}`));
						return;
					}

					mkdirSync(path.dirname(profileOutput), { recursive: true });
					const rustupResult = spawnSync("rustup", ["which", "llvm-profdata"], {
						encoding: "utf8",
						shell: true
					});
					let llvmProfdata = rustupResult.status === CARGO_SAFELY_EXIT_CODE
						? rustupResult.stdout.trim()
						: undefined;
					if (!llvmProfdata) {
						const sysrootResult = spawnSync("rustc", ["--print", "sysroot"], {
							encoding: "utf8",
							shell: true
						});
						const hostResult = spawnSync("rustc", ["-vV"], {
							encoding: "utf8",
							shell: true
						});
						const host = hostResult.stdout.match(/^host: (.+)$/m)?.[1];
						if (sysrootResult.status === CARGO_SAFELY_EXIT_CODE && host) {
							const sysrootProfdata = path.join(
								sysrootResult.stdout.trim(),
								"lib",
								"rustlib",
								host,
								"bin",
								"llvm-profdata"
							);
							if (existsSync(sysrootProfdata)) {
								llvmProfdata = sysrootProfdata;
							}
						}
					}
					llvmProfdata ||= "llvm-profdata";
					const mergeResult = spawnSync(llvmProfdata, ["merge", "-o", profileOutput, ...profrawFiles], {
						stdio: "inherit",
						shell: true
					});
					if (mergeResult.status !== CARGO_SAFELY_EXIT_CODE) {
						resolve(mergeResult.status || 1);
						return;
					}
				}
			}
			resolve(code);
		});
	});
}
