// HMR runners check behavior; omit duplicate full-runtime snapshots.
export default () => !process.env.RSPACK_HOT_TEST;
