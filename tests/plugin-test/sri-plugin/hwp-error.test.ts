/**
 * Copyright (c) 2015-present, Waysact Pty Ltd
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import { resolve } from "path";
import { experiments } from "@rspack/core";
import { runRspack } from "./test-utils";

const { SubresourceIntegrityPlugin } = experiments;

jest.mock("html-webpack-plugin");

describe("sri-plugin/hwp-error", () => {
  test("error when loading html-webpack-plugin", async () => {
    await expect(
      runRspack({
        entry: resolve(__dirname, "./__fixtures__/simple-project/src/"),
        plugins: [new SubresourceIntegrityPlugin({
          htmlPlugin: "html-webpack-plugin"
        })],
      })
    ).rejects.toThrow("bogus hwp accessed");
  });
});

