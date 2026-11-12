export const load = () => import(/* webpackChunkName: "leaf" */ './leaf').then((leaf) => leaf.default);
