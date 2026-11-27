/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
export default {
  checkCache({ namespace, cold, restored }) {
    if (namespace !== '|Compilation/modules') return;

    // finishMake forces a.js to rebuild on every compiler after initial cache reuse.
    expect(cold).toEqual({ hits: 0, misses: 3, reused: 0, stores: 2 });
    expect(restored).toEqual({ hits: 3, misses: 0, reused: 2, stores: 1 });
    return true;
  },
};
