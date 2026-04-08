export * from "./lib/index";

if (!!(process.env.NODE_ENV !== "production")) {
    console.log("This is a side effect");
}
