import { registerServerReference } from "react-server-dom-rspack/server";
export async function actionA() {
    return 'hello from actionA';
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    actionA
]);
registerServerReference(actionA, "001a7ae7c5453e05b873be6860e0e6fb009b74b9b83a8e69fdfa29c74ff981854e", null);
