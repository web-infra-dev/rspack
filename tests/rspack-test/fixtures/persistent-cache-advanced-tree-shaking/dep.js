import { tracker } from "./tracker";

export function check() {
  tracker.push("impure");
}
