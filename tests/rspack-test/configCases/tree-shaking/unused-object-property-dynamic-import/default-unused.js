import { effects } from "./module";

export default {
  id: effects.push("unused default"),
  loader: () =>
    import(
      /* webpackChunkName: "default-unused" */ "./default-unused-lazy"
    ),
};
