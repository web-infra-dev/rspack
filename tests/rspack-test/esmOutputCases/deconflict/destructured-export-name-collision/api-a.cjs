class Command {
	constructor(name) {
		this.name = `a:${name}`;
	}
}

module.exports = {
	createCommand: name => new Command(name),
	Command
};
