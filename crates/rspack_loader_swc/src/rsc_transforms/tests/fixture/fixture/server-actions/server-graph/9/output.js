// app/send.ts
import { registerServerReference } from "react-server-dom-rspack/server";
async function foo() {}
export { foo };
async function bar() {}
export { bar as baz };
async function qux() {}
export { qux as default };
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    foo,
    bar,
    qux
]);
registerServerReference(foo, "0060fbe7f56026259f29d3e16651fdc9021e1d9bc070b2e0df61cccc2bbb4fe5a7", null);
registerServerReference(bar, "00d1504e6981c398d4b8059c5f13757c752c2119409a6bc7200b5624514a5ced93", null);
registerServerReference(qux, "00629c080094d3e42e3f04da3fc764e18c5905c42019036be89980b7d901dc746d", null);
