import web from "./Foo.web";
import { name, square } from "./Foo.native";
console.log("square:", square);
export default web as name;
