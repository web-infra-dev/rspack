export { connect } from "./connect";
// external value for runtime a
import { value } from "./value";

// add side effect
console.log.bind(console);

export function Provide() {
  value;
}
