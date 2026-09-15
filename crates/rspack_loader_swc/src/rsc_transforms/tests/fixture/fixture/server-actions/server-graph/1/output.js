import { registerServerReference } from "react-server-dom-rspack/server";
import { encryptActionBoundArgs, decryptActionBoundArgs } from "react-server-dom-rspack/server";
import { Button } from 'components';
import deleteFromDb from 'db';
export const $$RSC_SERVER_ACTION_0 = async function deleteItem($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1] = await decryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", $$ACTION_CLOSURE_BOUND);
    await deleteFromDb($$ACTION_ARG_0);
    await deleteFromDb($$ACTION_ARG_1);
};
registerServerReference($$RSC_SERVER_ACTION_0, "40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export function Item({ id1, id2 }) {
    var deleteItem = $$RSC_SERVER_ACTION_0.bind(null, encryptActionBoundArgs("40d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", id1, id2));
    return /*#__PURE__*/ React.createElement(Button, {
        action: deleteItem
    }, "Delete");
}
export const $$RSC_SERVER_ACTION_1 = async function action($$ACTION_CLOSURE_BOUND) {
    var [$$ACTION_ARG_0, $$ACTION_ARG_1] = await decryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", $$ACTION_CLOSURE_BOUND);
    console.log($$ACTION_ARG_0);
    console.log($$ACTION_ARG_1);
};
registerServerReference($$RSC_SERVER_ACTION_1, "40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export default function Home() {
    const info = {
        name: 'John',
        test: 'test'
    };
    const action = $$RSC_SERVER_ACTION_1.bind(null, encryptActionBoundArgs("40be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", info.name, info.test));
    return null;
}
