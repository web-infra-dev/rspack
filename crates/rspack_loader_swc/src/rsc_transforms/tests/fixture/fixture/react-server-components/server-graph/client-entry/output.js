// This is a comment.
import { registerClientReference } from "react-server-dom-rspack/server";
export default registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "default");
export const a = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "a");
export const b = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "b");
export const c = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "c");
export const f = registerClientReference(function() {
    throw new Error("Attempted to call the default export of \"/some-project/src/some-file.js\" from the server, but it's on the client. It's not possible to invoke a client function from the server, it can only be rendered as a Component or passed to props of a Client Component.");
}, "/some-project/src/some-file.js", "f");
