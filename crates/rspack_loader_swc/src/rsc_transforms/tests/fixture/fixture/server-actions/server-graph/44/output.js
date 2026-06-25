import { registerServerReference } from "react-server-dom-rspack/server";
import { Form } from 'components';
export const $$RSC_SERVER_ACTION_0 = async function foo() {
    return 'declarator arrow function expression';
};
registerServerReference($$RSC_SERVER_ACTION_0, "00d7300b3a327ac30acf7f3331539670b902e44a4fa90021b9bb918e6741e18e80", null);
export const $$RSC_SERVER_ACTION_1 = async function bar() {
    return 'function declaration';
};
registerServerReference($$RSC_SERVER_ACTION_1, "00be8d67bb01f9fc1199f26e7d4bbc812d5446fe5337d20608749bb1de5f3e8caa", null);
export const $$RSC_SERVER_ACTION_2 = async function action() {
    return 'arrow function expression';
};
registerServerReference($$RSC_SERVER_ACTION_2, "006bee4aba7e4600414a77a62a5fdf44cd90de849943a217961e7b080db8db5aa4", null);
export const $$RSC_SERVER_ACTION_3 = async function action() {
    return 'anonymous function expression';
};
registerServerReference($$RSC_SERVER_ACTION_3, "00e688a8a841ab032e37e1a7bdc968ef0ebb18456250007692193a66472be5964f", null);
export const $$RSC_SERVER_ACTION_4 = async function baz() {
    return 'named function expression';
};
registerServerReference($$RSC_SERVER_ACTION_4, "00572e186264d2d532a250e5acf6b198b7ca6d98529584db7bbcdde20450a0f203", null);
export default function Page() {
    const foo = $$RSC_SERVER_ACTION_0;
    var bar = $$RSC_SERVER_ACTION_1;
    return /*#__PURE__*/ React.createElement(React.Fragment, null, /*#__PURE__*/ React.createElement(Form, {
        action: foo
    }), /*#__PURE__*/ React.createElement(Form, {
        action: bar
    }), /*#__PURE__*/ React.createElement(Form, {
        action: $$RSC_SERVER_ACTION_2
    }), /*#__PURE__*/ React.createElement(Form, {
        action: $$RSC_SERVER_ACTION_3
    }), /*#__PURE__*/ React.createElement(Form, {
        action: $$RSC_SERVER_ACTION_4
    }));
}
