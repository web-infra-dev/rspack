import { effects } from "./module";

export default {
  id: effects.push("used default"),
  loader: () =>
    import(/* webpackChunkName: "default-used" */ "./default-used-lazy"),
};
