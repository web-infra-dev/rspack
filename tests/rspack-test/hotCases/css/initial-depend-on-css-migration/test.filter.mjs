// HMR runners check behavior; omit duplicate full-runtime snapshots.
export default ({ target }) => target === 'web' && !process.env.RSPACK_HOT_TEST;
