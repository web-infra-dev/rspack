import { registerServerReference } from "react-server-dom-rspack/server";
export const dec = async (value)=>{
    return value - 1;
};
// Test case for https://github.com/vercel/next.js/issues/54655
export default dec;
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    dec
]);
registerServerReference(dec, "40629c080094d3e42e3f04da3fc764e18c5905c42019036be89980b7d901dc746d", null);
