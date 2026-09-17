/** @type {import("@rspack/core").Configuration} */
export default {
  target: ['node', 'es2020'],
  output: {
    environment: {
      // Our target supports `globalThis`, but for test purposes we set it to `false`
      globalThis: false,
    },
  },
};
