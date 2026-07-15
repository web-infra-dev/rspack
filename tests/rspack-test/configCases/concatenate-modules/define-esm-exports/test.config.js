const fs = require("fs");
const path = require("path");

const IDENTIFIER = "[A-Za-z_$][\\w$]*";
const escapeRegExp = (value) => value.replace(/[|\\{}()[\]^$+*?.]/g, "\\$&");

function countTargetCalls(source, runtime, method, target) {
  const pattern = new RegExp(
    `${escapeRegExp(runtime)}\\.${method}\\(\\s*${escapeRegExp(target)}\\s*(?=[,)])`,
    "g",
  );
  return source.match(pattern)?.length || 0;
}

function extractExportsTarget(source, runtime) {
  const markerIndex = source.indexOf("// EXPORTS");
  if (markerIndex < 0) {
    throw new Error("Unable to find concatenated exports marker");
  }

  const pattern = new RegExp(
    `${escapeRegExp(runtime)}\\.(?:esm|d)\\(\\s*(${IDENTIFIER})\\s*,\\s*\\{`,
  );
  const match = pattern.exec(source.slice(markerIndex));
  if (!match) {
    throw new Error("Unable to identify concatenated exports argument");
  }
  return match[1];
}

function extractNamespaceTarget(source) {
  const marker = "// NAMESPACE OBJECT: ./namespace.js";
  const markerIndex = source.indexOf(marker);
  if (markerIndex < 0) {
    throw new Error("Unable to find captured namespace marker");
  }

  const match = new RegExp(`var\\s+(${IDENTIFIER})\\s*=\\s*\\{\\};`).exec(
    source.slice(markerIndex),
  );
  if (!match) {
    throw new Error("Unable to identify captured namespace object");
  }
  return match[1];
}

function getTargetCallCounts(source, runtime, target) {
  return {
    esm: countTargetCalls(source, runtime, "esm", target),
    r: countTargetCalls(source, runtime, "r", target),
    N: countTargetCalls(source, runtime, "N", target),
    d: countTargetCalls(source, runtime, "d", target),
  };
}

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  findBundle() {
    return ["./main.js"];
  },
  afterExecute(options) {
    const mainSource = fs.readFileSync(
      path.resolve(options.output.path, "main.js"),
      "utf-8",
    );
    const concatenatedSource = fs.readFileSync(
      path.resolve(options.output.path, "concatenated.js"),
      "utf-8",
    );

    const isRspackRuntime = mainSource.includes("var __rspack_context={};");
    const runtime = isRspackRuntime
      ? "__rspack_context"
      : "__webpack_require__";
    const exportsTarget = extractExportsTarget(concatenatedSource, runtime);
    const namespaceTarget = extractNamespaceTarget(concatenatedSource);

    expect(exportsTarget).not.toBe(namespaceTarget);
    expect({
      exports: getTargetCallCounts(
        concatenatedSource,
        runtime,
        exportsTarget,
      ),
      namespace: getTargetCallCounts(
        concatenatedSource,
        runtime,
        namespaceTarget,
      ),
    }).toEqual({
      exports: { esm: 1, r: 0, N: 0, d: 0 },
      namespace: { esm: 1, r: 0, N: 0, d: 0 },
    });

    const namespaceDeclaration = `var ${namespaceTarget} = {};`;
    const namespaceCall = new RegExp(
      `${escapeRegExp(runtime)}\\.esm\\(\\s*${escapeRegExp(namespaceTarget)}\\s*,`,
    ).exec(concatenatedSource);
    expect(namespaceCall).not.toBeNull();
    expect(namespaceCall.index).toBeGreaterThan(
      concatenatedSource.indexOf(namespaceDeclaration),
    );

    if (isRspackRuntime) {
      expect(mainSource).toContain(
        "__rspack_context.esm = defineEsmExports;",
      );
      expect(mainSource).toContain("var defineEsmExports =");
      expect(mainSource).toContain("var makeNamespaceObject =");
      expect(mainSource).toContain("var definePropertyGetters =");
      expect(mainSource).toContain("makeNamespaceObject(exports);");
      expect(mainSource).toContain(
        "definePropertyGetters(exports, getters, values);",
      );
    } else {
      expect(mainSource).toContain("__webpack_require__.esm =");
      expect(mainSource).toContain("__webpack_require__.r =");
      expect(mainSource).toContain("__webpack_require__.d =");
      expect(mainSource).toContain("__webpack_require__.r(exports);");
      expect(mainSource).toContain(
        "__webpack_require__.d(exports, getters, values);",
      );
    }
  },
};
