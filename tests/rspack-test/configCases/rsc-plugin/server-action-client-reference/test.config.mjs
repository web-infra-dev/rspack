export default {
  findBundle: () => ['server-main.js'],
  checkCache({ compilerIndex, namespace, cold, restored }) {
    // Three rounds add a client module. The back edge can widen Outer's export
    // set, replacing the temporary RSC entry once more.
    return (
      compilerIndex === 1 &&
      namespace === '|Compilation/modules' &&
      restored.misses <= 4 &&
      restored.reused === restored.hits &&
      restored.reused === cold?.stores
    );
  },
};
