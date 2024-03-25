"use strict";

// eslint-disable-next-line node/no-unpublished-require
const CLI = require("../../packages/webpack-cli/lib/webpack-cli");

// eslint-disable-next-line node/no-unpublished-require
const stripAnsi = require("strip-ansi");

const readlineQuestionMock = jest.fn();

jest.mock("readline", () => {
  return {
    createInterface: jest.fn().mockReturnValue({
      question: readlineQuestionMock,
      close: jest.fn().mockImplementation(() => undefined),
    }),
  };
});

const spawnMock = jest.fn();

jest.mock("cross-spawn", () => ({ sync: spawnMock }));

describe("doInstall", () => {
  let cli;
  let getDefaultPackageManagerSpy;

  beforeEach(() => {
    cli = new CLI();

    getDefaultPackageManagerSpy = jest.spyOn(cli, "getDefaultPackageManager");
  });

  afterEach(() => {
    jest.clearAllMocks();

    getDefaultPackageManagerSpy.mockRestore();
  });

  it("should prompt to install using npm if npm is package manager", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("y"));
    getDefaultPackageManagerSpy.mockReturnValue("npm");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'npm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("npm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should prompt to install using yarn if yarn is package manager", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("y"));
    getDefaultPackageManagerSpy.mockReturnValue("yarn");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'yarn add -D test-package')",
    );

    // install the package using yarn
    expect(spawnMock.mock.calls[0][0]).toEqual("yarn");
    expect(spawnMock.mock.calls[0][1]).toEqual(["add", "-D", "test-package"]);
  });

  it("should prompt to install using pnpm if pnpm is package manager", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("y"));
    getDefaultPackageManagerSpy.mockReturnValue("pnpm");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'pnpm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("pnpm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should support pre message", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("y"));
    getDefaultPackageManagerSpy.mockReturnValue("npm");

    const preMessage = jest.fn();
    const installResult = await cli.doInstall("test-package", { preMessage });

    expect(installResult).toBe("test-package");
    expect(preMessage.mock.calls.length).toEqual(1);
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'npm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("npm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should prompt to install and install using 'y'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("y"));
    getDefaultPackageManagerSpy.mockReturnValue("npm");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'npm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("npm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should prompt to install and install using 'yes'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("yes"));
    getDefaultPackageManagerSpy.mockReturnValue("npm");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'npm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("npm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should prompt to install and install using 'yEs'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("yEs"));
    getDefaultPackageManagerSpy.mockReturnValue("npm");

    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBe("test-package");
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    expect(spawnMock.mock.calls.length).toEqual(1);
    expect(stripAnsi(readlineQuestionMock.mock.calls[0][0])).toContain(
      "Would you like to install 'test-package' package? (That will run 'npm install -D test-package')",
    );

    // install the package using npm
    expect(spawnMock.mock.calls[0][0]).toEqual("npm");
    expect(spawnMock.mock.calls[0][1]).toEqual(["install", "-D", "test-package"]);
  });

  it("should not install if install is not confirmed", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("n"));

    const mockExit = jest.spyOn(process, "exit").mockImplementation(() => {});
    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBeUndefined();
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    // runCommand should not be called, because the installation is not confirmed
    expect(spawnMock.mock.calls.length).toEqual(0);
    expect(mockExit.mock.calls[0][0]).toEqual(2);

    mockExit.mockRestore();
  });

  it("should not install if install is not confirmed using 'n'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("n"));

    const mockExit = jest.spyOn(process, "exit").mockImplementation(() => {});
    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBeUndefined();
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    // runCommand should not be called, because the installation is not confirmed
    expect(spawnMock.mock.calls.length).toEqual(0);
    expect(mockExit.mock.calls[0][0]).toEqual(2);

    mockExit.mockRestore();
  });

  it("should not install if install is not confirmed using 'no'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("no"));

    const mockExit = jest.spyOn(process, "exit").mockImplementation(() => {});
    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBeUndefined();
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    // runCommand should not be called, because the installation is not confirmed
    expect(spawnMock.mock.calls.length).toEqual(0);
    expect(mockExit.mock.calls[0][0]).toEqual(2);

    mockExit.mockRestore();
  });

  it("should not install if install is not confirmed using 'no'", async () => {
    readlineQuestionMock.mockImplementation((_questionTest, cb) => cb("No"));

    const mockExit = jest.spyOn(process, "exit").mockImplementation(() => {});
    const installResult = await cli.doInstall("test-package");

    expect(installResult).toBeUndefined();
    expect(readlineQuestionMock.mock.calls.length).toEqual(1);
    // runCommand should not be called, because the installation is not confirmed
    expect(spawnMock.mock.calls.length).toEqual(0);
    expect(mockExit.mock.calls[0][0]).toEqual(2);

    mockExit.mockRestore();
  });
});
