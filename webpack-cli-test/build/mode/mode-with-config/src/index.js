console.log("Ichigo");

if (process.env.NODE_ENV === "production") {
    console.log("production mode");
} else if (process.env.NODE_ENV === "development") {
    console.log("development mode");
} else {
    console.log("none mode");
}
