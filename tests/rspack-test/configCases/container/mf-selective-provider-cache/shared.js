export default {
  id: module.id,
  lazy: () => import("./lazy").then((m) => m.id),
};
