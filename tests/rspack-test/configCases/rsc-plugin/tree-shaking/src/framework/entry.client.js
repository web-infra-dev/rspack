// In a real app this entry would consume the RSC payload and hydrate.
// This file exists mainly to mirror the typical split of RSC/SSR/client entries.

export const loadClientModule = async (chunkId, moduleId) => {
    await __webpack_chunk_load__(chunkId);
    const mod = __webpack_require__(moduleId);
    return Object.keys(mod);
}
