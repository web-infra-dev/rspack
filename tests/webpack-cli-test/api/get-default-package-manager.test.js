const fs = require("fs");
const path = require("path");
// eslint-disable-next-line node/no-unpublished-require
const CLI = require("../../packages/webpack-cli/lib/webpack-cli");

const syncMock = jest.fn(() => {
  return {
    stdout: "1.0.0",
  };
});
jest.setMock("cross-spawn", {
  sync: syncMock,
});

describe("getPackageManager", () => {
  let cli;

  const testYarnLockPath = path.resolve(__dirname, "test-yarn-lock");
  const testNpmLockPath = path.resolve(__dirname, "test-npm-lock");
  const testPnpmLockPath = path.resolve(__dirname, "test-pnpm-lock");
  const testNpmAndPnpmPath = path.resolve(__dirname, "test-npm-and-pnpm");
  const testNpmAndYarnPath = path.resolve(__dirname, "test-npm-and-yarn");
  const testYarnAndPnpmPath = path.resolve(__dirname, "test-yarn-and-pnpm");
  const testAllPath = path.resolve(__dirname, "test-all-lock");
  const noLockPath = path.resolve(__dirname, "no-lock-files");

  const cwdSpy = jest.spyOn(process, "cwd");

  beforeAll(() => {
    // package-lock.json is ignored by .gitignore, so we simply
    // write a lockfile here for testing
    if (!fs.existsSync(testNpmLockPath)) {
      fs.mkdirSync(testNpmLockPath);
    }

    fs.writeFileSync(path.resolve(testNpmLockPath, "package-lock.json"), "");
    fs.writeFileSync(path.resolve(testNpmAndPnpmPath, "package-lock.json"), "");
    fs.writeFileSync(path.resolve(testNpmAndYarnPath, "package-lock.json"), "");
    fs.writeFileSync(path.resolve(testAllPath, "package-lock.json"), "");
  });

  beforeEach(() => {
    cli = new CLI();

    syncMock.mockClear();
  });

  it("should find yarn.lock", () => {
    cwdSpy.mockReturnValue(testYarnLockPath);

    expect(cli.getDefaultPackageManager()).toEqual("yarn");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should find package-lock.json", () => {
    cwdSpy.mockReturnValue(testNpmLockPath);

    expect(cli.getDefaultPackageManager()).toEqual("npm");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should find pnpm-lock.yaml", () => {
    cwdSpy.mockReturnValue(testPnpmLockPath);

    expect(cli.getDefaultPackageManager()).toEqual("pnpm");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should prioritize npm over pnpm", () => {
    cwdSpy.mockReturnValue(testNpmAndPnpmPath);

    expect(cli.getDefaultPackageManager()).toEqual("npm");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should prioritize npm over yarn", () => {
    cwdSpy.mockReturnValue(testNpmAndYarnPath);

    expect(cli.getDefaultPackageManager()).toEqual("npm");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should prioritize yarn over pnpm", () => {
    cwdSpy.mockReturnValue(testYarnAndPnpmPath);

    expect(cli.getDefaultPackageManager()).toEqual("yarn");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should prioritize npm with many lock files", () => {
    cwdSpy.mockReturnValue(testAllPath);

    expect(cli.getDefaultPackageManager()).toEqual("npm");
    expect(syncMock.mock.calls.length).toEqual(0);
  });

  it("should prioritize global npm over other package managers", () => {
    cwdSpy.mockReturnValue(noLockPath);

    expect(cli.getDefaultPackageManager()).toEqual("npm");
    expect(syncMock.mock.calls.length).toEqual(1);
  });

  it("should throw error if no package manager is found", () => {
    syncMock.mockImplementation(() => {
      throw new Error();
    });
    const mockExit = jest.spyOn(process, "exit").mockImplementation(() => {});
    // Do not print warning in CI
    const consoleMock = jest.spyOn(console, "error").mockImplementation(() => {});

    expect(cli.getDefaultPackageManager()).toBeFalsy();
    expect(mockExit).toBeCalledWith(2);
    expect(consoleMock).toHaveBeenCalledTimes(1);
    expect(syncMock.mock.calls.length).toEqual(3); // 3 calls for npm, yarn and pnpm
  });
});
