export const effects = [];
effects.push("module evaluated");

export const unusedLoader = () =>
  import(/* webpackChunkName: "dead" */ "./async");

export const live = "live";
