// Added to the loader list by the `beforeLoaders` tap, never by the configuration.
// It appends an export so that the bundle shows whether the tap's change to the
// loader list actually took effect on this build.
module.exports = function (source) {
  return `${source}\nexport const tag = 'tagged-by-beforeLoaders';\n`;
};
