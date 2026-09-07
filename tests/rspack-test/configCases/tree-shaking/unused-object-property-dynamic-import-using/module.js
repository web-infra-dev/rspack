export let disposalImport;

using resource = {
  [Symbol.dispose]: () => {
    disposalImport = import(
      /* webpackChunkName: "dispose" */ "./dispose"
    );
  },
};
