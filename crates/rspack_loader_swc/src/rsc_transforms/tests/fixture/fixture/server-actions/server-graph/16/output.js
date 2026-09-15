import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
import { Button } from 'components';
import deleteFromDb from 'db';
const v1 = 'v1';
export const $$RSC_SERVER_ACTION_0 = async function deleteItem($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1] = await decryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0);
    await deleteFromDb(v1);
    await deleteFromDb($$ACTION_ARG_1);
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function Item({ id1, id2 }) {
    const v2 = id2;
    const deleteItem = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", id1, v2));
    return /*#__PURE__*/ React.createElement(Button, {
        action: deleteItem
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_1 = async function g($$ACTION_CLOSURE_BOUND, y, ...z) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("7fbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    return $$ACTION_ARG_0 + y + z[0];
};
registerServerReference($$RSC_SERVER_ACTION_1, "7fbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
let f = (x)=>{
    var g = $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("7fbe8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", x));
};
export const $$RSC_SERVER_ACTION_2 = async function f($$ACTION_CLOSURE_BOUND, y, ...z) {
    var [$$ACTION_ARG_0] = await decryptActionBoundArgs("7f6bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", $$ACTION_CLOSURE_BOUND);
    return $$ACTION_ARG_0 + y + // can't be a `ts-expect-error` because the type of `z` changes to `any` in the output
    // and it stops being an error
    //
    // @ts-ignore: incompatible argument types
    z[0];
};
registerServerReference($$RSC_SERVER_ACTION_2, "7f6bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", null);
const g = (x)=>{
    f = $$RSC_SERVER_ACTION_2.bind(null, encryptActionBoundArgs("7f6bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", x));
};
