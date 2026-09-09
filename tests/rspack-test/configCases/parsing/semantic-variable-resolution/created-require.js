import { createRequire } from 'module';

it('keeps outer tag invalidation after exiting nested scopes', () => {
  let load = createRequire(import.meta.url);
  expect(load('./value')).toBe('bundled');
  function reset() {
    {
      load = request => request;
    }
    expect(load('./missing-inner')).toBe('./missing-inner');
  }
  reset();
  expect(load('./missing-outer')).toBe('./missing-outer');
});

it('does not carry a reassigned local tag into the enclosing binding', () => {
  const load = createRequire(import.meta.url);
  (function () {
    let load = createRequire(import.meta.url);
    load = request => request;
    expect(load('./missing-local')).toBe('./missing-local');
  })();
  expect(load('./value')).toBe('bundled');
});

it('keeps an alias snapshot when its source binding is reassigned', () => {
  let load = createRequire(import.meta.url);
  const alias = load;
  load = request => request;
  expect(load('./missing-source')).toBe('./missing-source');
  expect(alias('./value')).toBe('bundled');
});

it('restores nested parameter aliases independently from their sources', () => {
  const load = createRequire(import.meta.url);
  (function (alias) {
    const snapshot = alias;
    (function (alias) {
      expect(alias('./missing-parameter')).toBe('parameter');
    })(() => 'parameter');
    expect(alias('./value')).toBe('bundled');
    expect(snapshot('./value')).toBe('bundled');
  })(load);
  expect(load('./value')).toBe('bundled');
});
