/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
export default {
  checkCache({ namespace, cold, restored }) {
    if (namespace === '|Compilation/modules') {
      // loader.mjs marks render.css non-cacheable (propagating to CssModule), and finishMake forces a rebuild.
      expect(cold).toEqual({ hits: 0, misses: 8, reused: 0, stores: 6 });
      expect(restored).toEqual({ hits: 8, misses: 0, reused: 4, stores: 2 });
      return true;
    }

    if (namespace === '|Compilation/codeGeneration') {
      // importModule runs render.css with "green" then rebuilds it with "red" in the same slot.
      expect(cold).toEqual({ hits: 2, misses: 9, reused: 2, stores: 9 });
      expect(restored).toEqual({ hits: 9, misses: 2, reused: 9, stores: 2 });
      return true;
    }
  },
};
