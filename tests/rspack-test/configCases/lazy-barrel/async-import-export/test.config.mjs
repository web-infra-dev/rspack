export default {
  moduleScope(scope) {
    // The VM runner has no native dynamic import callback. Use a real host import.
    scope.testImport = (specifier) => import(specifier);
  },
};
