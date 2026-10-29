import store from "./store";
import "./entry";

it("should catch up before running a newer sibling of an unchanged chunk", async () => {
  await NEXT_BUILD();
  const stats = await NEXT_BUILD();
  await module.hot.check(true);
  await store.pending;
  expect(store.value).toBe(3);
  expect(__webpack_hash__).toBe(stats.hash);
  const repeated = await import(/* webpackChunkName: "page" */ "./page");
  expect(repeated.default).toBe(3);
});
