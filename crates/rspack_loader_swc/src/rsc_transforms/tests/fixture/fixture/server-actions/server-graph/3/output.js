// app/send.ts
import { registerServerReference } from "react-server-dom-rspack/server";
export async function myAction(a, b, c) {
    console.log('a');
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    myAction
]);
registerServerReference(myAction, "705c6b7a2e607afe6379ab0577ea157f1e81b7925ac0b5c06598a704d31997233b", null);
