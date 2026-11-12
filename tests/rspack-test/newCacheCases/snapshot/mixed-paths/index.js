const fileDependencies = {
  immutable: require('./consumer-immutable'),
  sibling: require('./consumer-sibling'),
  immutableRegexp: require('./consumer-immutable-regexp'),
  override: require('./consumer-override'),
  unmanagedRegexp: require('./consumer-unmanaged-regexp'),
  unmanagedPrefix: require('./consumer-unmanaged-prefix'),
  allPaths: require('./consumer-all-paths'),
  managed: require('./consumer-managed'),
  managedString: require('./consumer-managed-string'),
  managedPrefix: require('./consumer-managed-prefix'),
  scoped: require('./consumer-scoped'),
  nested: require('./consumer-nested'),
  context: require('./consumer-context'),
  missingName: require('./consumer-missing-name'),
  unmatched: require('./consumer-unmatched'),
};
const contextDependencies = {
  immutable: require('./consumer-immutable?context'),
  sibling: require('./consumer-sibling?context'),
  immutableRegexp: require('./consumer-immutable-regexp?context'),
  override: require('./consumer-override?context'),
  unmanagedRegexp: require('./consumer-unmanaged-regexp?context'),
  unmanagedPrefix: require('./consumer-unmanaged-prefix?context'),
  allPaths: require('./consumer-all-paths?context'),
  managed: require('./consumer-managed?context'),
  managedString: require('./consumer-managed-string?context'),
  managedPrefix: require('./consumer-managed-prefix?context'),
  scoped: require('./consumer-scoped?context'),
  nested: require('./consumer-nested?context'),
  context: require('./consumer-context?context'),
  missingName: require('./consumer-missing-name?context'),
  unmatched: require('./consumer-unmatched?context'),
};

it('should apply webpack snapshot path classification', async () => {
  // Every data.txt changes on every restart, independently of package metadata.
  const current = String(COMPILER_INDEX + 1);
  const expected = {
    // Immutable strings and regexps ignore content and package version changes.
    // immutableRegexp also matches managedPaths, so immutable must take precedence.
    immutable: '1',
    sibling: '1',
    immutableRegexp: '1',
    // Unmanaged strings and regexps override immutable and managed classifications.
    // allPaths matches all three options and must still track content changes.
    override: current,
    unmanagedRegexp: current,
    unmanagedPrefix: current,
    allPaths: current,
    // Managed strings and regexps ignore content changes until name@version changes.
    // The nearest named package wins even when an outer package version changes.
    managed: COMPILER_INDEX < 2 ? '1' : '3',
    managedString: COMPILER_INDEX < 2 ? '1' : '3',
    // String prefixes also match sibling directory names for all three options.
    managedPrefix: '1',
    // The scoped package has no version and changes its name on the first restart.
    scoped: COMPILER_INDEX === 0 ? '1' : '2',
    nested: COMPILER_INDEX === 0 ? '1' : '2',
    // Missing package.json or missing/empty name falls back to normal snapshots within context.
    context: current,
    missingName: current,
    // Paths outside the configured prefixes and regexps use normal snapshots.
    unmatched: current,
  };
  expect(fileDependencies).toEqual(expected);
  expect(contextDependencies).toEqual(expected);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
