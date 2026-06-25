/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    library: {
      type: 'jsonp',
      name: 'MyJsonpCallback',
    },
  },
};
