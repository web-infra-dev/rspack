/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'browserslist: kaios 3.0',
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('Test', (compilation) => {
        expect(compilation.outputOptions.environment).toMatchInlineSnapshot(`
          Object {
            arrowFunction: true,
            asyncFunction: true,
            bigIntLiteral: true,
            computedProperty: true,
            const: true,
            destructuring: true,
            document: true,
            dynamicImport: true,
            dynamicImportInWorker: true,
            forOf: true,
            globalThis: true,
            importMetaDirnameAndFilename: false,
            logicalAssignment: false,
            methodShorthand: true,
            module: true,
            nodePrefixForCoreModules: false,
            optionalChaining: true,
            templateLiteral: true,
          }
        `);
        expect(compilation.options.externalsPresets).toMatchInlineSnapshot(`
          Object {
            electron: false,
            electronMain: false,
            electronPreload: false,
            electronRenderer: false,
            node: false,
            nwjs: false,
            web: true,
          }
        `);
      });
    },
  ],
};
