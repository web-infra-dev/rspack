// Codegen target (NOT executed) for the build-resolved mock identity.
//
// Every generated `rstest_mock`/`rstest_unmock` call carries a trailing
// `{o, r}` argument: `o` is the absolute path of THIS file (the declaring
// module), `r` the build-resolved target — an absolute path for a bundled
// module, the external request for an external, or `null` when unresolved.

// Bundled relative target -> `r` is src/dep.js's absolute path.
rs.mock('./dep.js', () => ({ value: 'MOCKED' }));

// Externalized builtin -> `r` is the external request spelling.
rs.mock('node:os', () => ({ hostname: () => 'MOCKED' }));

// Unresolvable package (rstest allows mocking missing modules) -> `r` is null.
rs.mock('missing-pkg-for-resolved-info', () => ({ value: 'MOCKED' }));

// 1-arg auto-mock (no `__mocks__` file): the identity rides the synthetic
// target dependency, after the request literal.
rs.mock('./autoDep.js');

// 1-arg unmock: the identity follows the request on the method call itself.
rs.unmock('./unmockDep.js');

export const keep = () => 'keep';
