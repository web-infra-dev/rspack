// Test file for running remote app in Node.js environment
import * as _ from "lodash-es";

console.log("Remote App - Testing lodash-es functions");

// Test data
const users = [
	{ name: "john doe", email: "john@example.com", role: "admin", age: 30 },
	{ name: "jane smith", email: "jane@example.com", role: "user", age: 25 },
	{ name: "bob wilson", email: "bob@example.com", role: "admin", age: 35 }
];

// Test groupBy
const groupedByRole = _.groupBy(users, "role");
console.log("Grouped users by role:", JSON.stringify(groupedByRole, null, 2));

// Test capitalize
const capitalized = users.map(u => ({
	...u,
	name: _.capitalize(u.name)
}));
console.log("Capitalized names:", capitalized.map(u => u.name).join(", "));

// Test omit
const publicUsers = users.map(u => _.omit(u, ["email"]));
console.log("Users without email:", JSON.stringify(publicUsers, null, 2));

// Test pick
const namesOnly = users.map(u => _.pick(u, ["name", "role"]));
console.log("Names and roles:", JSON.stringify(namesOnly, null, 2));

// Test throttle (just verify it creates a function)
const throttled = _.throttle(() => console.log("Throttled!"), 1000);
console.log("Throttle function created:", typeof throttled === "function");

// Test debounce (just verify it creates a function)
const debounced = _.debounce(() => console.log("Debounced!"), 500);
console.log("Debounce function created:", typeof debounced === "function");

console.log("Remote App tests completed successfully!");
