export default {
  findBundle: () => ['server-main.js'],
  checkCache({ compilerIndex, namespace, restored }) {
    // The first client entry only discovers actions. The final entry replaces it
    // before the persistent cache is written.
    return (
      compilerIndex === 1 &&
      namespace === '|Compilation/modules' &&
      restored.misses === 1 &&
      restored.reused === restored.hits
    );
  },
};
