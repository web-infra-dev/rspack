export default async function App() {
  return await import("./m1").then(({ m1 }) => m1());
};
