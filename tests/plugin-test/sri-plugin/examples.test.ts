/**
 * Copyright (c) 2015-present, Waysact Pty Ltd
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import { readdirSync, readFileSync } from "fs";
import spawn from "cross-spawn";
import { join } from "path";
import { rimraf } from "rimraf";

jest.unmock("html-webpack-plugin");

jest.setTimeout(120000);

const DISABLED_CASES = [
  "hwp-externals", // TODO: html-webpack-externals-plugin failed
  "lazy-hashes-cycles", // TODO: support hashLoading: "lazy"
  "lazy-hashes-group", // TODO: support hashLoading: "lazy"
  "lazy-hashes-multiple-parents", // TODO: support hashLoading: "lazy"
  "lazy-hashes-simple", // TODO: support hashLoading: "lazy"
  "no-error-invalid-config", // TODO: support compilation.hooks.renderManifest
  "sourcemap-code-splitting", // TODO: sourcemap hash content failed
  "webpack-assets-manifest", // TODO: support webpack-assets-manifest plugin
  "webpack-fix-style-only-entries", // TODO: support webpack-assets-manifest plugin
  "wsi-test-helper.js",
];

const DISABLED_RSPACK_CASES = [
  ...DISABLED_CASES,
];

const exampleDir = join(__dirname, "examples");
const rspackCliBin = join(__dirname, "../../../packages/rspack-cli/bin/rspack.js");

function createTestCases(type: "webpack" | "rspack") {
   readdirSync(exampleDir)
    .filter(i => !(type === "rspack" ? DISABLED_RSPACK_CASES : DISABLED_CASES).includes(i))
    .forEach((example) => {
      const exampleDirectory = join(exampleDir, example);
      const configFile = "webpack.config.js";
      const configContent = readFileSync(join(exampleDirectory, configFile));
      if (type === "rspack" && !configContent.includes("createHtmlPlugin")) {
        return;
      }
      test.concurrent(`${example}/${type}`, async () => {
        rimraf.sync(join(exampleDirectory, "dist", type));
        await new Promise<void>((resolve, reject) => {
          const stdout: string[] = [];
          const stderr: string[] = [];
          // CHANGED: run rspack and remove coverage
          const cmd = spawn(
            "node",
            [rspackCliBin, "build", "-c", configFile],
            {
              cwd: exampleDirectory,
              stdio: "pipe",
              env: {
                HTML_PLUGIN: type,
                ...process.env
              }
            }
          );
          cmd.stdout?.on("data", (data) => {
            stdout.push(data);
          });
          cmd.stderr?.on("data", (data) => {
            stderr.push(data);
          });
          cmd.on("exit", (code) => {
            if (code === 0) {
              resolve();
            } else {
              reject(
                new Error(
                  `child process exited with code ${code}: ${stdout.join(
                    ""
                  )} ${stderr.join("")}`
                )
              );
            }
          });
          cmd.on("error", reject);
        });
        await new Promise((resolve) => setTimeout(resolve, 0));
      });
    });
}

describe("sri-plugin/examples/webpack", () => {
  createTestCases("webpack");
});

describe("sri-plugin/examples/rspack", () => {
  createTestCases("rspack");
});
