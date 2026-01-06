import foo25, {
  foo14,
  foo15,
  foo16,
  foo17,
  foo18,
  foo19,
  foo20,
  foo21,
  foo22,
  foo23,
  foo24,
} from './decl'

// all invoke should have no side effects, and its return value is not used,
// so they all should be removed
foo14();
foo15();
foo16();
foo17();
foo18();
foo19();
foo20();
foo21();
foo22();
foo23();
foo24();
foo25();