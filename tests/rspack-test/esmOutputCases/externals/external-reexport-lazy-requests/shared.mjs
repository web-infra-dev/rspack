export const load = () => import(/* webpackChunkName: "both" */ './b.mjs');
export const other = () => import(/* webpackChunkName: "both" */ './a.mjs');
