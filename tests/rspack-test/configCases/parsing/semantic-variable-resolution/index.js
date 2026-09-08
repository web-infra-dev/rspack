require('./created-require');

it('keeps lexical parameters separate from free require aliases', () => {
  (function (require) {
    expect(require('./missing-parameter')).toBe('parameter');
  })(() => 'parameter');
  (function (alias) {
    expect(alias('./value')).toBe('bundled');
  })(require);
  expect(require('./value')).toBe('bundled');
});

it('resolves bindings in loop, catch, method and default-parameter scopes', () => {
  for (let require of [() => 'loop']) {
    expect(require('./missing-loop')).toBe('loop');
  }
  try {
    throw () => 'catch';
  } catch (require) {
    expect(require('./missing-catch')).toBe('catch');
  }
  const object = {
    method(require = () => 'default') {
      return require('./missing-method');
    },
  };
  expect(object.method()).toBe('default');
  const Named = class require {
    static self() {
      return require;
    }
  };
  expect(Named.self()).toBe(Named);
  expect(require('./value')).toBe('bundled');
});

it('keeps hoisted declarations available before and outside their blocks', () => {
  const local = () => 'hoisted';
  require = local;
  expect(require('./missing-hoisted-var')).toBe('hoisted');
  if (true) {
    var require;
  }
});

it('preserves sloppy block function declaration lookup', () => {
  if (true) {
    function require() {
      return 'block function';
    }
  }
  expect(require('./missing-block-function')).toBe('block function');
});

it('keeps suppressed require.ensure parameters on the dynamic path', done => {
  require.ensure([], function (require) {
    expect(require('./value')).toBe('bundled');
    done();
  });
});

it('does not mix temporary AST IDs with module symbols', () => {
  // Both replacements are separately parsed trees. Free names use the
  // insertion scope, while declarations inside a replacement stay local.
  const require = () => 'caller';
  expect(CALLER_EXPRESSION).toBe('caller');
  expect(LOCAL_EXPRESSION).toBe('fragment');
  expect(CALLER_EXPRESSION).toBe('caller');
});

it('restores the caller environment after walking replacement scopes', () => {
  expect(LOCAL_EXPRESSION).toBe('fragment');
  expect(require('./context/' + LOCAL_EXPRESSION + '.js')).toBe('fragment dependency');
  expect(require('./value')).toBe('bundled');
});
