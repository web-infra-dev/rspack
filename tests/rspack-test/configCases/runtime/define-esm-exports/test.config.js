const fs = require("fs");
const path = require("path");

const escapeRegExp = (value) => value.replace(/[|\\{}()[\]^$+*?.]/g, "\\$&");

function extractModuleSource(source, moduleId) {
  const moduleHeader = new RegExp(
    "^(?:\\/\\*\\*\\*\\/\\s*)?" +
      escapeRegExp(JSON.stringify(moduleId)) +
      "(?::|\\([^\\n]*\\)\\s*\\{)",
    "m",
  );
  const headerMatch = moduleHeader.exec(source);
  if (!headerMatch) {
    throw new Error("Unable to find module definition for " + moduleId);
  }

  const searchStart = headerMatch.index + headerMatch[0].length;
  const remainingSource = source.slice(searchStart);
  const nextModuleHeader = remainingSource.search(
    /^(?:\/\*\*\*\/\s*)?"(?:[^"\\]|\\.)+"(?::|\([^\n]*\)\s*\{)/m,
  );
  const runtimeSection = remainingSource.search(
    /^(?:\/\/\s*|\/\*{6}\/\s*\/\*\s*)(?:webpack|rspack)\/runtime\//m,
  );
  const boundaries = [nextModuleHeader, runtimeSection].filter(
    (index) => index >= 0,
  );
  if (boundaries.length === 0) {
    throw new Error("Unable to find the end of module definition for " + moduleId);
  }

  return source.slice(
    headerMatch.index,
    searchStart + Math.min(...boundaries),
  );
}

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    const libSource = extractModuleSource(source, "./lib.js");

    if (source.includes("var __rspack_context={};")) {
      expect(
        libSource.match(
          /__rspack_context\.esm\(__rspack_exports, \{/g,
        ),
      ).toHaveLength(1);
      expect(libSource).not.toContain(
        "__rspack_context.N(__rspack_exports);",
      );
      expect(libSource).not.toContain(
        "__rspack_context.d(__rspack_exports, {",
      );
      expect(source).toContain("__rspack_context.esm = defineEsmExports;");
      const markerCall = source.indexOf("makeNamespaceObject(exports);");
      const gettersCall = source.indexOf(
        "definePropertyGetters(exports, getters, values);",
      );
      expect(markerCall).toBeGreaterThan(-1);
      expect(gettersCall).toBeGreaterThan(markerCall);
    } else {
      expect(
        libSource.match(
          /__webpack_require__\.esm\(__webpack_exports__, \{/g,
        ),
      ).toHaveLength(1);
      expect(libSource).not.toContain(
        "__webpack_require__.r(__webpack_exports__);",
      );
      expect(libSource).not.toContain(
        "__webpack_require__.d(__webpack_exports__, {",
      );
      expect(source).toContain("__webpack_require__.esm =");
      const markerCall = source.indexOf("__webpack_require__.r(exports);");
      const gettersCall = source.indexOf(
        "__webpack_require__.d(exports, getters, values);",
      );
      expect(markerCall).toBeGreaterThan(-1);
      expect(gettersCall).toBeGreaterThan(markerCall);
    }
  },
};
