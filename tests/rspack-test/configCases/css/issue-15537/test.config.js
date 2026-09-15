const fs = require("fs");
const path = require("path");
const vm = require("vm");

module.exports = {
  findBundle: () => [],
  validate(stats, _stderr, options) {
    const config = Array.isArray(options) ? options[0] : options;
    const source = fs.readFileSync(
      path.resolve(config.output.path, "bundle0.js"),
      "utf-8",
    );
    const context = vm.createContext({});

    vm.runInContext(source, context);

    expect(typeof context.cssClassName).toBe("string");

    const statsJson = stats.toJson({
      modules: true,
      nestedModules: true,
    });
    const modules =
      statsJson.modules ??
      statsJson.children?.flatMap((child) => child.modules ?? []) ??
      [];
    const concatenatedModule = modules.find((module) =>
      module.modules?.some((innerModule) =>
        innerModule.identifier.endsWith("style.css"),
      ),
    );

    expect(concatenatedModule).toBeDefined();
  },
};
