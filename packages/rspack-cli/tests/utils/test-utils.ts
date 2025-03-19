/* eslint-disable node/no-unpublished-require */

"use strict";

const os = require("os");
const { stripVTControlCharacters: stripAnsi } = require("node:util");
const path = require("path");
const net = require("node:net");
const fs = require("fs");
const execa = require("execa");
const { exec } = require("child_process");
const { node: execaNode } = execa;
const { Writable } = require("readable-stream");
const concat = require("concat-stream");

const RSPACK_PATH = path.resolve(__dirname, "../../bin/rspack.js");
const ENABLE_LOG_COMPILATION = process.env.ENABLE_PIPE || false;
const isWindows = process.platform === "win32";

function getInternalIpV4(): string | undefined {
	const nets = os.networkInterfaces();
	for (const name of Object.keys(nets)) {
		for (const net of nets[name] ?? []) {
			if (net.family === "IPv4" && !net.internal) {
				return net.address;
			}
		}
	}
	return undefined;
}

function getInternalIpV6(): string | undefined {
	const nets = os.networkInterfaces();
	for (const name of Object.keys(nets)) {
		for (const net of nets[name] ?? []) {
			if (net.family === "IPv6" && !net.internal) {
				return net.address;
			}
		}
	}
	return undefined;
}

const hyphenToUpperCase = name => {
	if (!name) {
		return name;
	}

	return name.replace(/-([a-z])/g, function (g) {
		return g[1].toUpperCase();
	});
};

const processKill = process => {
	if (isWindows) {
		exec("taskkill /pid " + process.pid + " /T /F");
	} else {
		process.kill();
	}
};

/**
 * Webpack CLI test runner.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Promise}
 */
const createProcess = (cwd, args, options, env) => {
	const { nodeOptions = [] } = options;
	const processExecutor = nodeOptions.length ? execaNode : execa;
	return processExecutor(RSPACK_PATH, args, {
		cwd: path.resolve(cwd),
		reject: false,
		stdio: ENABLE_LOG_COMPILATION ? "inherit" : "pipe",
		maxBuffer: Infinity,
		env: { RSPACK_CLI_HELP_WIDTH: 1024, ...env },
		...options
	});
};

/**
 * Run the rspack CLI for a test case.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Promise}
 */
const run = async (cwd, args: string[] = [], options = {}, env = {}) => {
	return createProcess(cwd, args, options, env);
};

/**
 * Run the rspack CLI for a test case and get process.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Promise}
 */
const runAndGetProcess = (cwd, args = [], options = {}, env = {}) => {
	return createProcess(cwd, args, options, env);
};

/**
 * Run the rspack CLI in watch mode for a test case.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Object} The rspack output or Promise when nodeOptions are present
 */
const runWatch = (
	cwd,
	args: string[] = [],
	options: Record<string, any> = {},
	env: Record<string, any> = {}
): any => {
	return new Promise((resolve, reject) => {
		const process = createProcess(cwd, args, options, env);
		const outputKillStr = options.killString || /rspack \d+\.\d+\.\d/;

		process.stdout.pipe(
			new Writable({
				write(chunk, encoding, callback) {
					const output = stripAnsi(chunk.toString("utf8"));

					if (outputKillStr.test(output)) {
						processKill(process);
					}

					callback();
				}
			})
		);

		process.stderr.pipe(
			new Writable({
				write(chunk, encoding, callback) {
					const output = stripAnsi(chunk.toString("utf8"));

					if (outputKillStr.test(output)) {
						processKill(process);
					}

					callback();
				}
			})
		);

		process
			.then(result => {
				resolve(result);
			})
			.catch(error => {
				reject(error);
			});
	});
};

/**
 * runPromptWithAnswers
 * @param {string} location location of current working directory
 * @param {string[]} args CLI args to pass in
 * @param {string[]} answers answers to be passed to stdout for inquirer question
 */
const runPromptWithAnswers = (location, args, answers) => {
	const process = runAndGetProcess(location, args);

	process.stdin.setDefaultEncoding("utf-8");

	const delay = 2000;
	let outputTimeout;
	let currentAnswer = 0;

	const writeAnswer = output => {
		if (!answers) {
			process.stdin.write(output);
			process.kill();

			return;
		}

		if (currentAnswer < answers.length) {
			process.stdin.write(answers[currentAnswer]);
			currentAnswer++;
		}
	};

	process.stdout.pipe(
		new Writable({
			write(chunk, encoding, callback) {
				const output = chunk.toString("utf8");

				if (output) {
					if (outputTimeout) {
						clearTimeout(outputTimeout);
					}

					// we must receive new stdout, then have 2 seconds
					// without any stdout before writing the next answer
					outputTimeout = setTimeout(() => {
						writeAnswer(output);
					}, delay);
				}

				callback();
			}
		})
	);

	return new Promise(resolve => {
		const obj: Record<string, any> = {};

		let stdoutDone = false;
		let stderrDone = false;

		const complete = () => {
			if (outputTimeout) {
				clearTimeout(outputTimeout);
			}

			if (stdoutDone && stderrDone) {
				process.kill("SIGKILL");
				resolve(obj);
			}
		};

		process.stdout.pipe(
			concat(result => {
				stdoutDone = true;
				obj.stdout = result.toString();

				complete();
			})
		);

		process.stderr.pipe(
			concat(result => {
				stderrDone = true;
				obj.stderr = result.toString();

				complete();
			})
		);
	});
};

const normalizeVersions = output => {
	return output.replace(
		/(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?/gi,
		"x.x.x"
	);
};

const normalizeCwd = output => {
	return output
		.replace(/\\/g, "/")
		.replace(new RegExp(process.cwd().replace(/\\/g, "/"), "g"), "<cwd>");
};

const normalizeError = output => {
	return output
		.replace(/SyntaxError: .+/, "SyntaxError: <error-message>")
		.replace(/\s+at .+(}|\)|\d)/gs, "\n    at stack");
};

const normalizeStdout = stdout => {
	if (typeof stdout !== "string") {
		return stdout;
	}

	if (stdout.length === 0) {
		return stdout;
	}

	let normalizedStdout = stripAnsi(stdout);
	normalizedStdout = normalizeCwd(normalizedStdout);
	normalizedStdout = normalizeVersions(normalizedStdout);
	normalizedStdout = normalizeError(normalizedStdout);

	return normalizedStdout;
};

const normalizeStderr = stderr => {
	if (typeof stderr !== "string") {
		return stderr;
	}

	if (stderr.length === 0) {
		return stderr;
	}

	let normalizedStderr = stripAnsi(stderr);
	normalizedStderr = normalizeCwd(normalizedStderr);

	const networkIPv4 = getInternalIpV4();

	if (networkIPv4) {
		normalizedStderr = normalizedStderr.replace(
			new RegExp(networkIPv4, "g"),
			"<network-ip-v4>"
		);
	}

	const networkIPv6 = getInternalIpV6();

	if (networkIPv6) {
		normalizedStderr = normalizedStderr.replace(
			new RegExp(networkIPv6, "g"),
			"<network-ip-v6>"
		);
	}

	normalizedStderr = normalizedStderr.replace(/:[0-9]+\//g, ":<port>/");

	if (!/On Your Network \(IPv6\)/.test(stderr)) {
		// Github Actions doesn't' support IPv6 on ubuntu in some cases
		normalizedStderr = normalizedStderr.split("\n");

		const ipv4MessageIndex = normalizedStderr.findIndex(item =>
			/On Your Network \(IPv4\)/.test(item)
		);

		if (ipv4MessageIndex !== -1) {
			normalizedStderr.splice(
				ipv4MessageIndex + 1,
				0,
				"<i> [rspack-dev-server] On Your Network (IPv6): http://[<network-ip-v6>]:<port>/"
			);
		}

		normalizedStderr = normalizedStderr.join("\n");
	}

	// the warning below is causing CI failure on some jobs
	if (/Gracefully shutting down/.test(stderr)) {
		normalizedStderr = normalizedStderr.replace(
			"\n<i> [rspack-dev-server] Gracefully shutting down. To force exit, press ^C again. Please wait...",
			""
		);
	}

	normalizedStderr = normalizeVersions(normalizedStderr);
	normalizedStderr = normalizeError(normalizedStderr);

	return normalizedStderr;
};

const readFile = (path, options = {}) =>
	new Promise((resolve, reject) => {
		fs.readFile(path, options, (err, stats) => {
			if (err) {
				reject(err);
			}
			resolve(stats);
		});
	});

const readdir = path =>
	new Promise((resolve, reject) => {
		fs.readdir(path, (err, stats) => {
			if (err) {
				reject(err);
			}
			resolve(stats);
		});
	});

// cSpell:ignore Symbhas, ABCDEFGHNR, Vfgcti
const urlAlphabet =
	"ModuleSymbhasOwnPr-0123456789ABCDEFGHNRVfgctiUvz_KqYTJkLxpZXIjQW";

const uuid = (size = 21) => {
	let id = "";
	let i = size;

	while (i--) {
		// `| 0` is more compact and faster than `Math.floor()`.
		id += urlAlphabet[(Math.random() * 64) | 0];
	}

	return id;
};

const uniqueDirectoryForTest = async () => {
	const result = path.resolve(os.tmpdir(), uuid());

	if (!fs.existsSync(result)) {
		fs.mkdirSync(result);
	}

	return result;
};

function isPortAvailable(port: number) {
	try {
		const server = net.createServer().listen(port);
		return new Promise(resolve => {
			server.on("listening", () => {
				server.close();
				resolve(true);
			});

			server.on("error", () => {
				resolve(false);
			});
		});
	} catch (err) {
		return false;
	}
}

const portMap = new Map();

// Available port ranges: 1024 ～ 65535
// `10080` is not available in macOS CI, `> 50000` get 'permission denied' in Windows.
// so we use `15000` ~ `45000`.
export async function getRandomPort(
	defaultPort = Math.ceil(Math.random() * 30000) + 15000
) {
	let port = defaultPort;
	while (true) {
		if (!portMap.get(port) && (await isPortAvailable(port))) {
			portMap.set(port, 1);
			return port;
		}
		port++;
	}
}

export {
	run,
	runAndGetProcess,
	runWatch,
	runPromptWithAnswers,
	isWindows,
	normalizeStderr,
	normalizeStdout,
	uniqueDirectoryForTest,
	readFile,
	readdir,
	hyphenToUpperCase,
	processKill
};
