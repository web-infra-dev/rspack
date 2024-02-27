/* eslint-disable node/no-unpublished-require */

"use strict";

const os = require("os");
const stripAnsi = require("strip-ansi");
const path = require("path");
const fs = require("fs");
const execa = require("execa");
const internalIp = require("internal-ip");
const { exec } = require("child_process");
const { node: execaNode } = execa;
const { Writable } = require("readable-stream");
const concat = require("concat-stream");
const { cli } = require("webpack");

const WEBPACK_PATH = path.resolve(__dirname, "../../packages/rspack-cli/bin/cli.js");
const ENABLE_LOG_COMPILATION = process.env.ENABLE_PIPE || false;
const isWindows = process.platform === "win32";

const hyphenToUpperCase = (name) => {
  if (!name) {
    return name;
  }

  return name.replace(/-([a-z])/g, function (g) {
    return g[1].toUpperCase();
  });
};

const processKill = (process) => {
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
const createProcess = (cwd, args, options) => {
  const { nodeOptions = [] } = options;
  const processExecutor = nodeOptions.length ? execaNode : execa;

  return processExecutor(WEBPACK_PATH, args, {
    cwd: path.resolve(cwd),
    reject: false,
    stdio: ENABLE_LOG_COMPILATION ? "inherit" : "pipe",
    maxBuffer: Infinity,
    env: { WEBPACK_CLI_HELP_WIDTH: 1024 },
    ...options,
  });
};

/**
 * Run the webpack CLI for a test case.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Promise}
 */
const run = async (cwd, args = [], options = {}) => {
  return createProcess(cwd, args, options);
};

/**
 * Run the webpack CLI for a test case and get process.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Promise}
 */
const runAndGetProcess = (cwd, args = [], options = {}) => {
  return createProcess(cwd, args, options);
};

/**
 * Run the webpack CLI in watch mode for a test case.
 *
 * @param {string} cwd The path to folder that contains test
 * @param {Array<string>} args Array of arguments
 * @param {Object<string, any>} options Options for tests
 * @returns {Object} The webpack output or Promise when nodeOptions are present
 */
const runWatch = (cwd, args = [], options = {}) => {
  return new Promise((resolve, reject) => {
    const process = createProcess(cwd, args, options);
    const outputKillStr = options.killString || /webpack \d+\.\d+\.\d/;
    const stdoutKillStr = options.stdoutKillStr;
    const stderrKillStr = options.stderrKillStr;

    let isStdoutDone = false;
    let isStderrDone = false;

    process.stdout.pipe(
      new Writable({
        write(chunk, encoding, callback) {
          const output = stripAnsi(chunk.toString("utf8"));

          if (stdoutKillStr && stdoutKillStr.test(output)) {
            isStdoutDone = true;
          } else if (!stdoutKillStr && outputKillStr.test(output)) {
            processKill(process);
          }

          if (isStdoutDone && isStderrDone) {
            processKill(process);
          }

          callback();
        },
      }),
    );

    process.stderr.pipe(
      new Writable({
        write(chunk, encoding, callback) {
          const output = stripAnsi(chunk.toString("utf8"));

          if (stderrKillStr && stderrKillStr.test(output)) {
            isStderrDone = true;
          } else if (!stderrKillStr && outputKillStr.test(output)) {
            processKill(process);
          }

          if (isStdoutDone && isStderrDone) {
            processKill(process);
          }

          callback();
        },
      }),
    );

    process
      .then((result) => {
        resolve(result);
      })
      .catch((error) => {
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

  const writeAnswer = (output) => {
    if (!answers) {
      process.stdin.write(output);
      processKill(process);

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
      },
    }),
  );

  return new Promise((resolve) => {
    const obj = {};

    let stdoutDone = false;
    let stderrDone = false;

    const complete = () => {
      if (outputTimeout) {
        clearTimeout(outputTimeout);
      }

      if (stdoutDone && stderrDone) {
        processKill(process);
        resolve(obj);
      }
    };

    process.stdout.pipe(
      concat((result) => {
        stdoutDone = true;
        obj.stdout = result.toString();

        complete();
      }),
    );

    process.stderr.pipe(
      concat((result) => {
        stderrDone = true;
        obj.stderr = result.toString();

        complete();
      }),
    );
  });
};

const normalizeVersions = (output) => {
  return output.replace(
    /(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?/gi,
    "x.x.x",
  );
};

const normalizeCwd = (output) => {
  return output
    .replace(/\\/g, "/")
    .replace(new RegExp(process.cwd().replace(/\\/g, "/"), "g"), "<cwd>");
};

const normalizeError = (output) => {
  return output
    .replace(/SyntaxError: .+/, "SyntaxError: <error-message>")
    .replace(/\s+at .+(}|\)|\d)/gs, "\n    at stack");
};

const normalizeStdout = (stdout) => {
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

const normalizeStderr = (stderr) => {
  if (typeof stderr !== "string") {
    return stderr;
  }

  if (stderr.length === 0) {
    return stderr;
  }

  let normalizedStderr = stripAnsi(stderr);
  normalizedStderr = normalizeCwd(normalizedStderr);

  const networkIPv4 = internalIp.v4.sync();

  if (networkIPv4) {
    normalizedStderr = normalizedStderr.replace(new RegExp(networkIPv4, "g"), "<network-ip-v4>");
  }

  const networkIPv6 = internalIp.v6.sync();

  if (networkIPv6) {
    normalizedStderr = normalizedStderr.replace(new RegExp(networkIPv6, "g"), "<network-ip-v6>");
  }

  normalizedStderr = normalizedStderr.replace(/:[0-9]+\//g, ":<port>/");

  if (!/On Your Network \(IPv6\)/.test(stderr)) {
    // Github Actions doesn't' support IPv6 on ubuntu in some cases
    normalizedStderr = normalizedStderr.split("\n");

    const ipv4MessageIndex = normalizedStderr.findIndex((item) =>
      /On Your Network \(IPv4\)/.test(item),
    );

    if (ipv4MessageIndex !== -1) {
      normalizedStderr.splice(
        ipv4MessageIndex + 1,
        0,
        "<i> [webpack-dev-server] On Your Network (IPv6): http://[<network-ip-v6>]:<port>/",
      );
    }

    normalizedStderr = normalizedStderr.join("\n");
  }

  // TODO remove me after drop old Node.js versions and update deps
  // Suppress warnings for Node.js version >= v21
  // [DEP0040] DeprecationWarning: The `punycode` module is deprecated. Please use a userland alternative instead.
  if (process.version.startsWith("v21")) {
    normalizedStderr = normalizedStderr
      .split("\n")
      .filter((line) => {
        return (
          !line.includes("DeprecationWarning: The `punycode` module is deprecated.") &&
          !line.includes("Use `node --trace-deprecation ...`")
        );
      })
      .join("\n");
  }

  // the warning below is causing CI failure on some jobs
  if (/Gracefully shutting down/.test(stderr)) {
    normalizedStderr = normalizedStderr.replace(
      "\n<i> [webpack-dev-server] Gracefully shutting down. To force exit, press ^C again. Please wait...",
      "",
    );
  }

  normalizedStderr = normalizeVersions(normalizedStderr);
  normalizedStderr = normalizeError(normalizedStderr);

  return normalizedStderr;
};

const getWebpackCliArguments = (startWith) => {
  if (typeof startWith === "undefined") {
    return cli.getArguments();
  }

  const result = {};

  for (const [name, value] of Object.entries(cli.getArguments())) {
    if (name.startsWith(startWith)) {
      result[name] = value;
    }
  }

  return result;
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

const readdir = (path) =>
  new Promise((resolve, reject) => {
    fs.readdir(path, (err, stats) => {
      if (err) {
        reject(err);
      }
      resolve(stats);
    });
  });

// cSpell:ignore Symbhas, ABCDEFGHNR, Vfgcti
const urlAlphabet = "ModuleSymbhasOwnPr-0123456789ABCDEFGHNRVfgctiUvz_KqYTJkLxpZXIjQW";

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

module.exports = {
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
  processKill,
  getWebpackCliArguments,
};
