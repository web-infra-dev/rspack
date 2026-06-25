import { registerServerReference } from "react-server-dom-rspack/server";
export const foo = async ()=>{};
const bar = async ()=>{};
export { bar };
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo,
    bar
]);
registerServerReference(foo, "0060fbe7f56026259f29d3e16651fdc9021e1d9bc070b2e0df61cccc2bbb4fe5a7", null);
registerServerReference(bar, "002f5ac883d87abb96a8f6d9197260b859ad0ad65a516d9d527f3b95b2445f8334", null);
