class Command {
	constructor(name) {
		this.name = `b:${name}`;
	}
}

module.exports = {
	createCommand: name => new Command(name),
	Command
};
