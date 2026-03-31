// Intentionally use an fs-shaped local name for the path namespace to trigger
// a cross-module external-name collision.
import * as external_fs_namespaceObject from "path";

const joinFn = Reflect.get(external_fs_namespaceObject, "join");

export { joinFn, external_fs_namespaceObject as pathNs };
