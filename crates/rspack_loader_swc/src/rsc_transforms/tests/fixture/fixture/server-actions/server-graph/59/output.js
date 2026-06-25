import { registerServerReference } from "react-server-dom-rspack/server";
import { db } from 'database';
export const createItem = async (title)=>{
    return new Promise((resolve, reject)=>{
        db.serialize(()=>{
            db.run(`INSERT INTO items (title) VALUES ($title)`, {
                $title: title
            }, function() {
                // arguments is allowed here
                const [err] = arguments;
                if (err) {
                    reject(err);
                }
                // this is allowed here
                resolve(this.lastID);
            });
        });
    });
};
export async function test() {
    const MyClass = class {
        x = 1;
        foo() {
            // this is allowed here
            return this.x;
        }
        bar = ()=>{
            // this is allowed here
            return this.x;
        };
    };
    const myObj = new MyClass();
    return myObj.foo() + myObj.bar();
}
import { ensureServerActions } from "react-server-dom-rspack/server";
ensureServerActions([
    createItem,
    test
]);
registerServerReference(createItem, "402d30911a9aa542c50ad8bce26f1e94e93a09c5355306b96b61e041ef63533f73", null);
registerServerReference(test, "007fe0f18c2be6d7c653e24e3c1396882858144f46e8c2088cecd17f132b1621f0", null);
