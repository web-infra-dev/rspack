import { registerServerReference } from "react-server-dom-rspack/server";
async function foo() {}
export { foo as '📙' };
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo
]);
registerServerReference(foo, "0019b8e24158aa2b08ce3ced6f30639c09984248f7b056cfd33ec17bd3c87c23d2", null);
