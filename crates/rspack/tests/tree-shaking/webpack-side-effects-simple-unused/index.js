import { log } from "pmodule/tracker";
import { x, z } from "pmodule";
import def from "pmodule";

def.should.be.eql("def");
x.should.be.eql("x");
z.should.be.eql("z");
log.should.be.eql(["b.js", "c.js", "index.js"]);
