export const loadFeature = () =>
  import(/* webpackChunkName: "feature" */ "./feature");

export const loadEagerFeature = () =>
  import(/* webpackMode: "eager" */ "./eager-feature");

export const live = "live";
