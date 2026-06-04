import { registerServerReference } from "react-server-dom-rspack/server";
import { Button } from 'components';
export const $$RSC_SERVER_ACTION_0 = async function myAction(a, b, c) {
    // comment
    'use strict';
    console.log('a');
};
registerServerReference($$RSC_SERVER_ACTION_0, "70d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
var myAction = $$RSC_SERVER_ACTION_0;
export default function Page() {
    return /*#__PURE__*/ React.createElement(Button, {
        action: myAction
    }, "Delete");
}
