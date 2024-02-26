export default function loader() {
	const callback = this.async();

	callback(
		null,
		`export default [
  [module.id, ".class-name-a {background: red;}", ""],
  [module.id, ".class-name-b {background: blue;}", ""],
];
  
export var cnA = () => "class-name-a";
export var cnB = () => "class-name-b";`
	);
}
