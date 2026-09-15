import { registerServerReference } from "react-server-dom-rspack/server";
export default async function foo() {}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo
]);
registerServerReference(foo, "00629c080094d3e42e3f04da3fc764e18c5905c42019036be89980b7d901dc746d", null);
