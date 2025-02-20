import {create} from "./base";
import {BuiltinPluginName} from "@rspack/binding";


export const ModuleInfoHeaderPlugin =  create(
    BuiltinPluginName.ModuleInfoHeaderPlugin,
    () => {},
    "compilation"
);
