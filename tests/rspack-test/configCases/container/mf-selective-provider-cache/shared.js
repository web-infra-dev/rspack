export default { lazy: () => import("./lazy").then((m) => m.default) };
