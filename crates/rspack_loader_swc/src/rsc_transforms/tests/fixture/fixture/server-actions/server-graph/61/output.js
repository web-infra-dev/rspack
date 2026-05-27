import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1] = await decryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    console.log($$ACTION_ARG_0.find((x)=>x === $$ACTION_ARG_1));
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function ComponentA({ list, y }) {
    return /*#__PURE__*/ React.createElement("form", {
        action: $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", list, y))
    }, /*#__PURE__*/ React.createElement("button", null, "submit"));
}
export const $$RSC_SERVER_ACTION_1 = async function action($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1] = await decryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    console.log($$ACTION_ARG_0.find(function(x) {
        return x === $$ACTION_ARG_1;
    }));
};
registerServerReference($$RSC_SERVER_ACTION_1, "40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export function ComponentB({ list, y }) {
    return /*#__PURE__*/ React.createElement("form", {
        action: $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", list, y))
    }, /*#__PURE__*/ React.createElement("button", null, "submit"));
}
