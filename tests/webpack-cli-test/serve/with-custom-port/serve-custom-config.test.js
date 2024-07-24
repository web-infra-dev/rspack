"use strict";

const path = require("path");
// eslint-disable-next-line node/no-unpublished-require
const getPort = require("get-port");
const { runWatch, normalizeStderr } = require("../../utils/test-utils");

const testPath = path.resolve(__dirname);

describe("serve with devServer in config", () => {
  let port;

  beforeEach(async () => {
    port = await getPort();
  });

  it("Should pick up the host and port from config", async () => {
    const { stdout, stderr } = await runWatch(testPath, ["serve"], {
      stdoutKillStr: /webpack \d+\.\d+\.\d/,
      stderrKillStr: /Content not from webpack is served from/,
    });

    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("HotModuleReplacementPlugin");
    expect(stdout).toContain("main.js");
  });

  it("Port flag should override the config port", async () => {
    const { stdout, stderr } = await runWatch(testPath, ["serve", "--port", port], {
      stdoutKillStr: /webpack \d+\.\d+\.\d/,
      stderrKillStr: /Content not from webpack is served from/,
    });

    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("HotModuleReplacementPlugin");
    expect(stdout).toContain("main.js");
  });

  it("Passing hot flag works alongside other server config", async () => {
    const { stdout, stderr } = await runWatch(testPath, ["serve", "--port", port, "--hot"], {
      stdoutKillStr: /webpack \d+\.\d+\.\d/,
      stderrKillStr: /Content not from webpack is served from/,
    });

    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).toContain("HotModuleReplacementPlugin");
    expect(stdout).toContain("main.js");
  });

  it("works fine when no-hot flag is passed alongside other server config", async () => {
    const { stdout, stderr } = await runWatch(testPath, ["serve", "--port", port, "--no-hot"], {
      stdoutKillStr: /webpack \d+\.\d+\.\d/,
      stderrKillStr: /Content not from webpack is served from/,
    });

    expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
    expect(stdout).not.toContain("HotModuleReplacementPlugin");
    expect(stdout).toContain("main.js");
  });
});
