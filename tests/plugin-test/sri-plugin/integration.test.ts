/**
 * Copyright (c) 2015-present, Waysact Pty Ltd
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import {
  Stats,
  StatsAsset,
  RspackOptionsNormalized,
} from "@rspack/core";
import { resolve } from "path";
import tmp from "tmp-promise";
import { experiments } from "@rspack/core";
import { runRspack } from "./test-utils";
import merge from "lodash/merge";
const { SubresourceIntegrityPlugin } = experiments;

jest.unmock("html-webpack-plugin");

async function runRspackForSimpleProject(
  options: Partial<RspackOptionsNormalized> = {}
): Promise<Stats> {
  const tmpDir = await tmp.dir({ unsafeCleanup: true });
  return await runRspack(
    merge(
      {
        mode: "production",
        output: { path: tmpDir.path, crossOriginLoading: "anonymous" },
        entry: resolve(__dirname, "./__fixtures__/simple-project/src/."),
        plugins: [new SubresourceIntegrityPlugin({
          htmlPlugin: require.resolve("html-webpack-plugin"),
        })],
      },
      options
    )
  );
}

describe("sri-plugin/integration", () => {
  test("enabled with webpack mode=production", async () => {
    const mainAsset = (await runRspackForSimpleProject())
      .toJson()
      .assets?.find((asset: StatsAsset) => asset.name === "main.js");
    expect(mainAsset).toBeDefined();
    expect(mainAsset?.["integrity"]).toMatch(/^sha384-\S+$/);
  });

  test("disabled with webpack mode=development", async () => {
    const mainAsset = (await runRspackForSimpleProject({ mode: "development" }))
      .toJson()
      .assets?.find((asset: StatsAsset) => asset.name === "main.js");
    expect(mainAsset).toBeDefined();
    expect(mainAsset?.["integrity"]).toBeUndefined();
  });

  const isHashWarning = (warning: Error) =>
    warning.message.match(/Use \[contenthash\] and ensure realContentHash/);

  test("warns when [fullhash] is used", async () => {
    const stats = await runRspackForSimpleProject({
      output: { filename: "[fullhash].js" },
    });

    expect(stats.compilation.warnings.find(isHashWarning)).toBeDefined();
  });

  test("warns when [contenthash] is used without realContentHash", async () => {
    const stats = await runRspackForSimpleProject({
      output: { filename: "[contenthash].js" },
      optimization: { realContentHash: false },
    });

    expect(stats.compilation.warnings.find(isHashWarning)).toBeDefined();
  });

  test("doesn't warn when [contenthash] is used with realContentHash", async () => {
    const stats = await runRspackForSimpleProject({
      output: { filename: "[contenthash].js" },
      optimization: { realContentHash: true },
    });

    expect(stats.compilation.warnings).toHaveLength(0);
  });

  test("doesn't warn with default options", async () => {
    const stats = await runRspackForSimpleProject();

    expect(stats.compilation.warnings).toHaveLength(0);
  });
});


