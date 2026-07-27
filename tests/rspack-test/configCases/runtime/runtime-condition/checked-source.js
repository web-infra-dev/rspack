const values = {
	checkedA: 1,
	checkedB: 2,
	checkedC: 3,
	checkedD: 4,
	checkedE: 5,
	checkedF: 6,
	checkedG: 7,
	checkedH: 8,
	setCheckedA(next) {
		Object(exports).checkedA = next;
	}
};

for (const key in values) Object(exports)[key] = values[key];
