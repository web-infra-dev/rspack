import { enable, mutable, off, on } from "./flags";
import { loadFeature } from "./feature";

it("uses imported constants as short-circuit dependency guards", async () => {
  const calls = [];
  const evaluate = value => {
    calls.push(value);
    return value;
  };

  const disabled = off && evaluate(true) && loadFeature();
  const enabled =
    on ||
    evaluate(false) ||
    import(/* webpackChunkName: "unreachable-or" */ "./value?or");

  expect(disabled).toBe(false);
  expect(enabled).toBe(true);
  expect(calls).toEqual([]);

  const afterEffect =
    evaluate(true) &&
    off &&
    import(/* webpackChunkName: "unreachable-after-effect" */ "./value?effect");
  expect(afterEffect).toBe(false);
  expect(calls).toEqual([true]);

  const live =
    (off || evaluate(true)) &&
    import(/* webpackChunkName: "reachable-mixed" */ "./value?live");

  expect((await live).value).toBe("reachable");
  expect(calls).toEqual([true, true]);

  enable();
  const fromMutable =
    mutable &&
    import(/* webpackChunkName: "reachable-mutable" */ "./value?mutable");
  expect((await fromMutable).value).toBe("reachable");
});
