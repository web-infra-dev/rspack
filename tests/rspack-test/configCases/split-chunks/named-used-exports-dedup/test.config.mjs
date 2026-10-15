export default {
  findBundle: (index) => {
    // The pruned variants deliberately leave bootstrap independent of common.
    if (index === 5) return [`prelude-${index}.js`, `bootstrap-${index}.js`];
    if (index === 6) return [`bootstrap-${index}.js`];
    return [`prelude-${index}.js`, `common-${index}.js`, `bootstrap-${index}.js`];
  },
};
