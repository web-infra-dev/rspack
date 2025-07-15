import { increment as inc, value } from "./counter";
import { print, resetCounter } from "./methods";

print(value);
inc();
inc();
inc();
print(value);
resetCounter();
print(value);

export { inc, print };
