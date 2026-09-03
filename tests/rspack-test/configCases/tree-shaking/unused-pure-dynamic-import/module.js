export const unusedLoader = () => import("./unused");

export const unusedEagerLoader = () =>
  import(/* webpackMode: "eager" */ "./unused-eager");

export const live = "live";
