class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.finishModules.tap('Test', () => {
        const entry = compilation.entries.get('main');
        const entryDependency = entry.dependencies[0];
        const entryModule = compilation.moduleGraph.getModule(entryDependency);

        const getDepsByRequest = (request) =>
          entryModule.dependencies.filter((dep) => dep.request === request);

        const staticDeps = getDepsByRequest('./data.json');
        expect(staticDeps.length).toBeGreaterThan(0);
        expect(staticDeps.every((dep) => dep.attributes?.type === 'json')).toBe(
          true,
        );

        const reexportDeps = getDepsByRequest('./reexport.json');
        expect(reexportDeps.length).toBeGreaterThan(0);
        expect(
          reexportDeps.every((dep) => dep.attributes?.type === 'json'),
        ).toBe(true);

        const plainDep = getDepsByRequest('./plain')[0];
        expect(plainDep.attributes).toBe(undefined);

        const blockDeps = entryModule.blocks.flatMap(
          (block) => block.dependencies,
        );
        const dynamicDep = blockDeps.find(
          (dep) => dep.request === './async.json',
        );
        expect(dynamicDep.attributes).toEqual({ type: 'json' });

        const dynamicConnection =
          compilation.moduleGraph.getConnection(dynamicDep);
        expect(dynamicConnection.dependency.attributes).toEqual({
          type: 'json',
        });

        const contextDep = entryModule.dependencies.find(
          (dep) => dep.type === 'import context',
        );
        expect(contextDep.attributes).toEqual({ type: 'json' });

        const contextModule =
          compilation.moduleGraph.getConnection(contextDep).module;
        const contextElementConnections = compilation.moduleGraph
          .getOutgoingConnections(contextModule)
          .filter((connection) => {
            return connection.dependency?.type === 'import() context element';
          });
        expect(contextElementConnections.length).toBeGreaterThan(0);
        expect(
          contextElementConnections.every((connection) => {
            return connection.dependency.attributes?.type === 'json';
          }),
        ).toBe(true);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  plugins: [new Plugin()],
};
