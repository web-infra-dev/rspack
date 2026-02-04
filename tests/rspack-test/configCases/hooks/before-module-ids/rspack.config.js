const { strict } = require("assert");

let hookCalled = false;
let modulesReceived = [];
let customIdsAssigned = new Map();
let modulePropertiesVerified = false;

class BeforeModuleIdsTestPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap("BeforeModuleIdsTestPlugin", compilation => {
      compilation.hooks.beforeModuleIds.tap("BeforeModuleIdsTestPlugin", modules => {
        hookCalled = true;
        for (const module of modules) {
          modulesReceived.push(module.identifier);
          if (module.identifier && module.identifier.includes("index.js")) {
            module.id = "custom-id-for-index";
            customIdsAssigned.set(module.identifier, module.id);

            // Verify that real module properties are accessible
            // libIdent should be available on the proxied module
            const libIdent = module.libIdent({ context: compiler.options.context });
            strict(typeof libIdent === 'string', `libIdent should return a string, got ${typeof libIdent}`);

            // userRequest should be available for NormalModule
            if (module.userRequest !== undefined) {
              strict(typeof module.userRequest === 'string', `userRequest should be a string, got ${typeof module.userRequest}`);
              modulePropertiesVerified = true;
            }

            // type should be available
            strict(typeof module.type === 'string', `type should be a string, got ${typeof module.type}`);
          }
        }
      });
    });

    compiler.hooks.done.tap("BeforeModuleIdsTestPlugin", stats => {
      const json = stats.toJson({ modules: true });
      strict(json.errors.length === 0, `Build had errors: ${JSON.stringify(json.errors)}`);
      strict(hookCalled, "beforeModuleIds hook should be called");
      strict(modulesReceived.length > 0, "beforeModuleIds should receive modules");
      strict(customIdsAssigned.size > 0, "Should have assigned at least one custom ID");
      strict(modulePropertiesVerified, "Should have verified module properties like userRequest");

      const indexModule = json.modules.find(m => m.identifier && m.identifier.includes("index.js"));
      strict(indexModule, "index.js module should exist in stats");
      strict(indexModule.id === "custom-id-for-index", `Module ID should be 'custom-id-for-index' but got '${indexModule.id}'`);
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  optimization: {
    concatenateModules: false,
  },
  plugins: [new BeforeModuleIdsTestPlugin()]
};
