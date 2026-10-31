import { defineConfig } from '@rspack/cli';
import type { NormalModule, Compiler } from '@rspack/core';
import { strict } from 'node:assert';

// beforeModuleIds receives proxies with a string identifier and writable id.
type ModuleWithId = Omit<NormalModule, 'identifier'> & {
  identifier: string;
  id: string | number | null;
};

let hookCalled = false;
const modulesReceived: string[] = [];
const customIdsAssigned = new Map<string, string | number>();
let modulePropertiesVerified = false;

class BeforeModuleIdsTestPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'BeforeModuleIdsTestPlugin',
      (compilation) => {
        compilation.hooks.beforeModuleIds.tap(
          'BeforeModuleIdsTestPlugin',
          (modules) => {
            hookCalled = true;
            for (const module of modules as unknown as Iterable<ModuleWithId>) {
              modulesReceived.push(module.identifier);
              if (module.identifier && module.identifier.includes('index.js')) {
                module.id = 'custom-id-for-index';
                customIdsAssigned.set(module.identifier, module.id);

                // Verify that real module properties are accessible
                // libIdent should be available on the proxied module
                const libIdent = module.libIdent({
                  context: compiler.options.context!,
                });
                strict(
                  typeof libIdent === 'string',
                  `libIdent should return a string, got ${typeof libIdent}`,
                );

                // userRequest should be available for NormalModule
                if (module.userRequest !== undefined) {
                  strict(
                    typeof module.userRequest === 'string',
                    `userRequest should be a string, got ${typeof module.userRequest}`,
                  );
                  modulePropertiesVerified = true;
                }

                // type should be available
                strict(
                  typeof module.type === 'string',
                  `type should be a string, got ${typeof module.type}`,
                );
              }

              if (module.identifier && module.identifier.includes('a.js')) {
                module.id = 42;
                customIdsAssigned.set(module.identifier, module.id);
              }

              if (module.identifier && module.identifier.includes('b.js')) {
                module.id = 0;
                customIdsAssigned.set(module.identifier, module.id);
              }
            }
          },
        );
      },
    );

    compiler.hooks.done.tap('BeforeModuleIdsTestPlugin', (stats) => {
      const json = stats.toJson({ modules: true });
      strict(
        json.errors!.length === 0,
        `Build had errors: ${JSON.stringify(json.errors)}`,
      );
      strict(hookCalled, 'beforeModuleIds hook should be called');
      strict(
        modulesReceived.length > 0,
        'beforeModuleIds should receive modules',
      );
      strict(
        customIdsAssigned.size > 0,
        'Should have assigned at least one custom ID',
      );
      strict(
        modulePropertiesVerified,
        'Should have verified module properties like userRequest',
      );

      const indexModule = json.modules!.find(
        (m) => m.identifier && m.identifier.includes('index.js'),
      );
      strict(indexModule, 'index.js module should exist in stats');
      strict(
        indexModule.id === 'custom-id-for-index',
        `Module ID should be 'custom-id-for-index' but got '${indexModule.id}'`,
      );

      const aModule = json.modules!.find(
        (m) => m.identifier && m.identifier.includes('a.js'),
      );
      strict(aModule, 'a.js module should exist in stats');
      strict(
        aModule.id === 42,
        `Module ID should be 42 (number) but got '${aModule.id}' (${typeof aModule.id})`,
      );

      const bModule = json.modules!.find(
        (m) => m.identifier && m.identifier.includes('b.js'),
      );
      strict(bModule, 'b.js module should exist in stats');
      strict(
        bModule.id === 0,
        `Module ID should be 0 (number) but got '${bModule.id}' (${typeof bModule.id})`,
      );
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  optimization: {
    concatenateModules: false,
    // Avoid the default export is inlined, which causes the module with custom ID not exist in stats.modules
    inlineExports: false,
  },
  plugins: [new BeforeModuleIdsTestPlugin()],
});
