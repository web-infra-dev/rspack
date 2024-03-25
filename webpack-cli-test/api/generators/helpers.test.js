const path = require("path");
// eslint-disable-next-line node/no-unpublished-require
const CLI = require("../../../packages/webpack-cli/lib/webpack-cli");

const utilsDirectory = {
  cli: "../../../packages/webpack-cli/lib/utils",
  generators: "../../../packages/generators/src/utils",
};

jest.mock(path.join(utilsDirectory.generators, "scaffold-utils"), () => ({
  List: jest.fn(),
}));

const { getInstaller, getTemplate, toKebabCase, toUpperCamelCase } = require(path.join(
  utilsDirectory.generators,
  "helpers",
));
const { List } = require(path.join(utilsDirectory.generators, "scaffold-utils"));

describe("helpers", () => {
  let cli;
  let getDefaultPackageManagerSpy;
  let context;

  beforeEach(() => {
    cli = new CLI();
    context = {
      prompt: () => {},
      supportedTemplates: ["default"],
      cli: cli,
    };
    getDefaultPackageManagerSpy = jest.spyOn(cli, "getDefaultPackageManager");
  });

  afterEach(() => {
    jest.clearAllMocks();

    getDefaultPackageManagerSpy.mockRestore();
  });

  it("toKebabCase() returns kebab case", () => {
    const kebabValue = toKebabCase("HtmlMinimizerPlugin");

    expect(kebabValue).toBe("html-minimizer-plugin");
  });

  it("toUpperCamelCase() returns camel case strings", () => {
    const camelCaseString = toUpperCamelCase("html-minimizer-webpack-plugin");

    expect(camelCaseString).toBe("HtmlMinimizerWebpackPlugin");
  });

  it("getInstaller() returns the available installer", async () => {
    // Multiple installers are not available
    getDefaultPackageManagerSpy.mockReturnValue(["npm"]);

    // User chose "pnpm"
    List.mockReturnValue({ packager: "npm" });

    // Invoke the helper function
    const installer = await getInstaller.call(context);

    expect(installer).toBe("npm");
  });

  it("getInstaller() invokes a List prompt if multiple installers are available", async () => {
    // Multiple installers are available
    getDefaultPackageManagerSpy.mockReturnValue(["npm", "yarn", "pnpm"]);

    // User chose "pnpm"
    List.mockReturnValue({ packager: "pnpm" });

    // Invoke the helper function
    const installer = await getInstaller.call(context);
    expect(installer).toBe("pnpm");
  });

  it("getTemplate() returns with the valid template", async () => {
    context.template = "default";

    // Invoke the helper function
    const template = await getTemplate.call(context);

    expect(template).toBe("default");
  });

  it("getTemplate() invokes a List prompt on supplying an invalid template", async () => {
    context.template = "unknown";

    // User chose "default"
    List.mockReturnValue({ selectedTemplate: "default" });

    const { logger } = cli;
    const loggerMock = jest.spyOn(logger, "warn").mockImplementation(() => {});
    // Invoke the helper function`
    const template = await getTemplate.call(context);

    expect(template).toBe("default");
    expect(loggerMock).toHaveBeenCalled();
  });
});
