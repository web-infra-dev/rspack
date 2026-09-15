import { registerServerReference } from "react-server-dom-rspack/server";
const foo = async function() {};
export default foo;
const bar = async function() {};
export { bar };
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo,
    bar
]);
registerServerReference(foo, "00629c080094d3e42e3f04da3fc764e18c5905c42019036be89980b7d901dc746d", null);
registerServerReference(bar, "002f5ac883d87abb96a8f6d9197260b859ad0ad65a516d9d527f3b95b2445f8334", null);
