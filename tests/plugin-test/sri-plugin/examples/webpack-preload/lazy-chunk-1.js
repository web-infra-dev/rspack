setTimeout(() => {
  import(/* webpackPreload: true */ "./lazy-chunk-2.js").then((mod) =>
    mod.test()
  );
}, 750);
