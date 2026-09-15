import { registerServerReference } from "react-server-dom-rspack/server";
export async function foo() {
    async function bar() {}
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo
]);
registerServerReference(foo, "0060fbe7f56026259f29d3e16651fdc9021e1d9bc070b2e0df61cccc2bbb4fe5a7", null);
