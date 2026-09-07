export const effects = [];

function createId(id) {
  effects.push(id);
  return id;
}

export const unusedArrowFeature = {
  id: createId("unused arrow"),
  loader: () => import(/* webpackChunkName: "unused" */ "./unused"),
};

export const unusedFunctionFeature = {
  id: createId("unused function"),
  loader: function () {
    return import(/* webpackChunkName: "unused" */ "./unused");
  },
};

export const unusedNestedFeature = {
  id: createId("unused nested loader"),
  loader: () => () =>
    import(/* webpackChunkName: "unused-nested" */ "./unused"),
};

export const usedFeature = {
  id: createId("used loader"),
  loader: () => import(/* webpackChunkName: "used" */ "./used"),
};

export const unusedEagerFeature = {
  id: createId("eager import"),
  value: import(/* webpackChunkName: "eager" */ "./eager"),
};

export const live = "live";
