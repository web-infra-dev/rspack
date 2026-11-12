import { registerServerReference } from "react-server-dom-rspack/server";
export const action = {
    async f (x) {
        ;
        (()=>{
            console.log(x);
        })();
    }
}.f;
export const action2 = new class X {
    async f(x) {
        ;
        (()=>{
            console.log(x);
        })();
    }
}().f;
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    action,
    action2
]);
registerServerReference(action, "7fa2596154c7aa07f03b5ef8eaef000c8894b51876f19ae00d947fa104c0d5857e", null);
registerServerReference(action2, "7fdeb5c9c3cddb80b9a5f7fa2f260fb2133ea392a753d1ce445f92e037e2a22e69", null);
