const { captureStdio } = require("@rspack/test-tools/helper/legacy/captureStdio");
const { createFsFromVolume, Volume } = require("memfs");
const fs = require("fs");

class MyPlugin {
	apply(compiler) {
		const logger = compiler.getInfrastructureLogger("MyPlugin");
		logger.time("Time");
		logger.group("Group");
		logger.error("Error");
		logger.warn("Warning");
		logger.info("Info");
		logger.log("Log");
		logger.debug("Debug");
		logger.groupCollapsed("Collapsed group");
		logger.log("Log inside collapsed group");
		logger.groupEnd();
		logger.groupEnd();
		logger.timeEnd("Time");
	}
}
const escapeAnsi = stringRaw =>
	stringRaw
		.replace(/\u001b\[1m\u001b\[([0-9;]*)m/g, "<CLR=$1,BOLD>")
		.replace(/\u001b\[1m/g, "<CLR=BOLD>")
		.replace(/\u001b\[39m\u001b\[22m/g, "</CLR>")
		.replace(/\u001b\[([0-9;]*)m/g, "<CLR=$1>");

let capture;
let nativeInfrastructureLogs;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description: "should log to the console (verbose)",
		options(context) {
			capture = captureStdio(process.stderr);
			return {
				context: context.getSource(),
				entry: "./a",
				plugins: [new MyPlugin()],
				infrastructureLogging: {
					level: "verbose"
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async check() {
			expect(capture.toString().replace(/[\d.]+ ms/, "X ms"))
				.toMatchInlineSnapshot(`
				<-> [MyPlugin] Group
				  <e> [MyPlugin] Error
				  <w> [MyPlugin] Warning
				  <i> [MyPlugin] Info
				      [MyPlugin] Log
				  <-> [MyPlugin] Collapsed group
				        [MyPlugin] Log inside collapsed group
				<t> [MyPlugin] Time: X ms
			`);
			capture.restore();
		}
	},
	{
		description: "should log to the console (debug mode)",
		options(context) {
			capture = captureStdio(process.stderr);
			return {
				context: context.getSource(),
				entry: "./a",
				plugins: [new MyPlugin()],
				infrastructureLogging: {
					level: "error",
					debug: /MyPlugin/
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async check() {
			expect(capture.toString().replace(/[\d.]+ ms/, "X ms"))
				.toMatchInlineSnapshot(`
			<-> [MyPlugin] Group
			  <e> [MyPlugin] Error
			  <w> [MyPlugin] Warning
			  <i> [MyPlugin] Info
			      [MyPlugin] Log
			      [MyPlugin] Debug
			  <-> [MyPlugin] Collapsed group
			        [MyPlugin] Log inside collapsed group
			<t> [MyPlugin] Time: X ms
		`);
			capture.restore();
		}
	},
	{
		description: "should log to the console (none)",
		options(context) {
			capture = captureStdio(process.stderr);
			return {
				context: context.getSource(),
				entry: "./a",
				plugins: [new MyPlugin()],
				infrastructureLogging: {
					level: "none"
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async check() {
			expect(capture.toString()).toMatchInlineSnapshot("");
			capture.restore();
		}
	},
	{
		description: "should log to the console with colors (verbose)",
		options(context) {
			capture = captureStdio(process.stderr);
			return {
				context: context.getSource(),
				entry: "./a",
				plugins: [new MyPlugin()],
				infrastructureLogging: {
					level: "verbose",
					colors: true
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async check() {
			expect(escapeAnsi(capture.toStringRaw()).replace(/[\d.]+ ms/, "X ms"))
				.toMatchInlineSnapshot(`
			<-> <CLR=36,BOLD>[MyPlugin] Group</CLR>
			  <e> <CLR=31,BOLD>[MyPlugin] Error</CLR>
			  <w> <CLR=33,BOLD>[MyPlugin] Warning</CLR>
			  <i> <CLR=32,BOLD>[MyPlugin] Info</CLR>
			      <CLR=BOLD>[MyPlugin] Log<CLR=22>
			  <-> <CLR=36,BOLD>[MyPlugin] Collapsed group</CLR>
			        <CLR=BOLD>[MyPlugin] Log inside collapsed group<CLR=22>
			<t> <CLR=35,BOLD>[MyPlugin] Time: X ms</CLR>
		`);
			capture.restore();
		}
	},
	{
		description: "should log to the console with colors (debug mode)",
		options(context) {
			capture = captureStdio(process.stderr);
			return {
				context: context.getSource(),
				entry: "./a",
				plugins: [new MyPlugin()],
				infrastructureLogging: {
					level: "error",
					debug: /MyPlugin/,
					colors: true
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async check() {
			expect(escapeAnsi(capture.toStringRaw()).replace(/[\d.]+ ms/, "X ms"))
				.toMatchInlineSnapshot(`
			<-> <CLR=36,BOLD>[MyPlugin] Group</CLR>
			  <e> <CLR=31,BOLD>[MyPlugin] Error</CLR>
			  <w> <CLR=33,BOLD>[MyPlugin] Warning</CLR>
			  <i> <CLR=32,BOLD>[MyPlugin] Info</CLR>
			      <CLR=BOLD>[MyPlugin] Log<CLR=22>
			      [MyPlugin] Debug
			  <-> <CLR=36,BOLD>[MyPlugin] Collapsed group</CLR>
			        <CLR=BOLD>[MyPlugin] Log inside collapsed group<CLR=22>
			<t> <CLR=35,BOLD>[MyPlugin] Time: X ms</CLR>
		`);
			capture.restore();
		}
	},
	{
		description:
			"should preserve native infrastructure logs emitted before the first compilation",
		options(context) {
			const cacheLocation = context.getDist("invalid-cache-location");
			fs.writeFileSync(cacheLocation, "not a directory");
			nativeInfrastructureLogs = [];
			return {
				context: context.getSource(),
				entry: "./a",
				experiments: {
					newCache: true
				},
				cache: {
					type: "persistent",
					storage: {
						type: "filesystem",
						location: cacheLocation
					}
				},
				infrastructureLogging: {
					level: "none"
				}
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.hooks.infrastructureLog.tap(
				"NativeInfrastructureLoggingTest",
				(name, type, args) => {
					nativeInfrastructureLogs.push({ name, type, args });
				}
			);
		},
		async check() {
			const findDatabaseWarning = () =>
				nativeInfrastructureLogs.find(
					log =>
						log.name === "rspack.cache.IdleFileCache" &&
						log.type === "warn" &&
						log.args[0].includes("Open cache from")
				);
			for (let i = 0; i < 50; i++) {
				if (findDatabaseWarning()) break;
				await new Promise(resolve => setTimeout(resolve, 20));
			}
			const warning = findDatabaseWarning();
			expect(warning).toBeTruthy();
			expect(warning.args[0]).toContain("invalid-cache-location");
		}
	}
];
