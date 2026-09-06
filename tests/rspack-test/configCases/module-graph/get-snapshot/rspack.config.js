const {
  CssExtractRspackPlugin,
  ModuleGraph,
  ModuleGraphConnection,
} = require('@rspack/core');

const PLUGIN_NAME = 'Test';

function decodeState(value) {
  switch (value) {
    case ModuleGraph.EDGE_STATE_INACTIVE:
      return false;
    case ModuleGraph.EDGE_STATE_ACTIVE:
      return true;
    case ModuleGraph.EDGE_STATE_CIRCULAR:
      return ModuleGraphConnection.CIRCULAR_CONNECTION;
    case ModuleGraph.EDGE_STATE_TRANSITIVE_ONLY:
      return ModuleGraphConnection.TRANSITIVE_ONLY;
    default:
      throw new Error(`unexpected edge state ${value}`);
  }
}

/**
 * `getSnapshot` must describe exactly the graph that the per-module and
 * per-connection getters describe. This walks both and compares them.
 */
function assertSnapshotMatchesGetters(compilation) {
  const moduleGraph = compilation.moduleGraph;
  const snapshot = moduleGraph.getSnapshot();
  const modules = Array.from(compilation.modules);

  expect(snapshot.exportsInfoAvailable).toBe(true);
  expect(snapshot.identifiers).toHaveLength(modules.length);
  expect(snapshot.nameForConditions).toHaveLength(modules.length);
  expect(snapshot.layers).toHaveLength(modules.length);
  expect(snapshot.edgeOffsets).toHaveLength(modules.length + 1);
  expect(snapshot.edgeOffsets[0]).toBe(0);

  const edgeCount = snapshot.edgeOffsets[modules.length];
  expect(snapshot.edgeTargets).toHaveLength(edgeCount);
  expect(snapshot.edgeRequests).toHaveLength(edgeCount);
  expect(snapshot.edgeStates).toHaveLength(edgeCount);

  const seen = new Set();
  let comparedEdges = 0;

  modules.forEach((module, index) => {
    expect(snapshot.identifiers[index]).toBe(module.identifier());
    expect(seen.has(snapshot.identifiers[index])).toBe(false);
    seen.add(snapshot.identifiers[index]);

    expect(snapshot.nameForConditions[index] ?? null).toBe(
      module.nameForCondition() ?? null,
    );
    expect(snapshot.layers[index] ?? null).toBe(module.layer ?? null);

    const connections = moduleGraph.getOutgoingConnectionsInOrder(module);
    const start = snapshot.edgeOffsets[index];
    const end = snapshot.edgeOffsets[index + 1];
    expect(end - start).toBe(connections.length);

    connections.forEach((connection, offset) => {
      const at = start + offset;
      const target = snapshot.edgeTargets[at];
      expect(
        target === ModuleGraph.NO_MODULE ? null : snapshot.identifiers[target],
      ).toBe(connection.module ? connection.module.identifier() : null);

      const request = snapshot.edgeRequests[at];
      const expectedRequest =
        typeof connection.dependency?.request === 'string'
          ? connection.dependency.request
          : undefined;
      expect(request === -1 ? undefined : snapshot.requests[request]).toBe(
        expectedRequest,
      );

      expect(decodeState(snapshot.edgeStates[at])).toBe(
        connection.getActiveState(undefined),
      );
      comparedEdges += 1;
    });
  });

  expect(comparedEdges).toBe(edgeCount);
  expect(comparedEdges).toBeGreaterThan(0);

  // The request table must be deduplicated. Assert that directly rather than
  // through a count inequality, which would also hold for a table that simply
  // happened to be short: no duplicate entries, and at least one entry that is
  // genuinely shared by more than one edge.
  expect(new Set(snapshot.requests).size).toBe(snapshot.requests.length);
  const requestUses = new Map();
  for (const index of snapshot.edgeRequests) {
    if (index !== -1) {
      requestUses.set(index, (requestUses.get(index) ?? 0) + 1);
    }
  }
  expect(Math.max(...requestUses.values())).toBeGreaterThan(1);

  // Entry roots must match `getConnection(entryDependency).module`.
  const expectedEntries = [];
  for (const entry of compilation.entries.values()) {
    for (const dependency of entry.dependencies) {
      const module = moduleGraph.getConnection(dependency)?.module;
      if (module) {
        expectedEntries.push(module.identifier());
      }
    }
  }
  expect(
    Array.from(snapshot.entryModules).map(
      (index) => snapshot.identifiers[index],
    ),
  ).toEqual(expectedEntries);
  expect(expectedEntries.length).toBeGreaterThan(0);

  // The fixture is built so that inactive and non-boolean states occur; a
  // snapshot that reported everything as active would pass a comparison that
  // only ever saw `true`.
  const states = new Set(Array.from(snapshot.edgeStates));
  expect(states.has(ModuleGraph.EDGE_STATE_ACTIVE)).toBe(true);
  expect(states.size).toBeGreaterThan(1);
}

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        assertSnapshotMatchesGetters(compilation);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
  },
  plugins: [new CssExtractRspackPlugin(), new Plugin()],
};
