import { Bench } from "tinybench";
import { withCodSpeed } from "@codspeed/tinybench-plugin";

function fibonacci(n) {
    if (n < 2) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const bench = withCodSpeed(new Bench());

bench
    .add("fibonacci10", () => {
        fibonacci(10);
    })
    .add("fibonacci15", () => {
        fibonacci(15);
    });

await bench.run();
console.table(bench.table());
