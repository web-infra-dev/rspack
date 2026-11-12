import { registerClientReference } from "react-server-dom-rspack/server";
export const foo = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/app/item.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/app/item.js", "foo");
