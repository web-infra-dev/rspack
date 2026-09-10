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

it('initializes replacement declarations before walking their scopes', () => {
  // A context request forces the retained replacement expression through the
  // walker. Missing requests must stay calls to the fragment's local bindings.
  expect(require('./context/' + SCOPED_EXPRESSION + '.js')).toBe('fragment dependency');
  expect(require('./context/' + SCOPED_EXPRESSION + '.js')).toBe('fragment dependency');
  expect(require('./value')).toBe('bundled');
});

it('resolves free replacement names in the caller rather than the fragment body', () => {
  const load = require;
  {
    const require = () => 'fragment';
    // The parameter default belongs to the parameter environment, not the body
    // that declares another require. Neither fragment may capture the other's IDs.
    expect(load('./context/' + DEFAULT_EXPRESSION + '.js')).toBe('fragment dependency');
    expect(load('./context/' + CALLER_EXPRESSION + '.js')).toBe('fragment dependency');
  }
  expect(load('./value')).toBe('bundled');
});

it('restores semantic contexts across nested replacement evaluation', () => {
  // Evaluating the outer definition evaluates another separately parsed definition.
  if (NESTED_CONDITION) {
    expect(require('./value')).toBe('bundled');
  } else {
    throw new Error('nested definition was not evaluated');
  }
  {
    const NESTED_LITERAL = 0;
    // A caller-local name must prevent the inner definition from being expanded.
    if (NESTED_CONDITION) throw new Error('replacement lost its caller binding');
  }
  expect(require('./value')).toBe('bundled');
});
