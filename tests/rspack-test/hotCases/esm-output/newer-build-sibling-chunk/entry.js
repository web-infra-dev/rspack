// Keep the module type unchanged while adding the import.
export {};
module.hot.accept();
---
import store from "./store";

store.pending = import(/* webpackChunkName: "page" */ "./page").then((m) => {
	store.value = m.default;
});
module.hot.accept();
