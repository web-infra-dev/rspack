import * as external_fs_namespaceObject from "path";

const joinFn = Reflect.get(external_fs_namespaceObject, "join");

export { joinFn, external_fs_namespaceObject as pathNs };
