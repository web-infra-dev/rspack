import store from "./store";
import "./entry";

it("should catch up before running a chunk that a newer build overwrote", async () => {
  await NEXT_BUILD();
  await NEXT_BUILD();
  await import.meta.webpackHot.check(true);
  await store.pending;
  expect(store.value).toBe(2);
});
