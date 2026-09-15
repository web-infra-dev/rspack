import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
import { Button } from 'components';
import deleteFromDb from 'db';
export const $$RSC_SERVER_ACTION_0 = async function deleteItem1($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0.id, $$ACTION_ARG_0?.foo, $$ACTION_ARG_0.bar.baz, $$ACTION_ARG_0[// @ts-expect-error: deliberate useless comma
    $$ACTION_ARG_1, $$ACTION_ARG_2]);
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function Item1(product, foo, bar) {
    const a = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", product, foo, bar));
    return /*#__PURE__*/ React.createElement(Button, {
        action: a
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_1 = async function deleteItem2($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0.id, $$ACTION_ARG_0?.foo, $$ACTION_ARG_0.bar.baz, $$ACTION_ARG_0[// @ts-expect-error: deliberate useless comma
    $$ACTION_ARG_1, $$ACTION_ARG_2]);
};
registerServerReference($$RSC_SERVER_ACTION_1, "40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export function Item2(product, foo, bar) {
    var deleteItem2 = $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", product, foo, bar));
    return /*#__PURE__*/ React.createElement(Button, {
        action: deleteItem2
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_2 = async function deleteItem3($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("406bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0.id, $$ACTION_ARG_0?.foo, $$ACTION_ARG_0.bar.baz, $$ACTION_ARG_0[// @ts-expect-error: deliberate useless comma
    $$ACTION_ARG_1, $$ACTION_ARG_2]);
};
registerServerReference($$RSC_SERVER_ACTION_2, "406bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", null);
export function Item3(product, foo, bar) {
    const deleteItem3 = $$RSC_SERVER_ACTION_2.bind(null, encryptActionBoundArgs("406bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", product, foo, bar));
    return /*#__PURE__*/ React.createElement(Button, {
        action: deleteItem3
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_3 = async function deleteItem4($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1, $$ACTION_ARG_2] = await decryptActionBoundArgs("40e688a8a841ab032e37e1a7bdc968ef0ebb18456250007692193a66472be5964f", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0.id, $$ACTION_ARG_0?.foo, $$ACTION_ARG_0.bar.baz, $$ACTION_ARG_0[// @ts-expect-error: deliberate useless comma
    $$ACTION_ARG_1, $$ACTION_ARG_2]);
};
registerServerReference($$RSC_SERVER_ACTION_3, "40e688a8a841ab032e37e1a7bdc968ef0ebb18456250007692193a66472be5964f", null);
export function Item4(product, foo, bar) {
    const deleteItem4 = $$RSC_SERVER_ACTION_3.bind(null, encryptActionBoundArgs("40e688a8a841ab032e37e1a7bdc968ef0ebb18456250007692193a66472be5964f", product, foo, bar));
    return /*#__PURE__*/ React.createElement(Button, {
        action: deleteItem4
    }, "Delete");
}
