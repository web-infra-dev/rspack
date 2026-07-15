const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );

    const libStart = source.indexOf('"./lib.js"');
    const libEnd = source.indexOf("\n\n},\n\n});", libStart);
    expect(libStart).toBeGreaterThan(-1);
    expect(libEnd).toBeGreaterThan(libStart);
    const libSource = source.slice(libStart, libEnd);

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
