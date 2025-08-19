import {
	addDays,
	format,
	isAfter,
	isBefore,
	parseISO,
	subDays
} from "date-fns";

// Export functions that use only a subset of date-fns
export const formatDate = (date, formatStr = "yyyy-MM-dd") => {
	return format(date, formatStr);
};

export const parseDate = dateString => {
	return parseISO(dateString);
};

export const addDaysToDate = (date, days) => {
	return addDays(date, days);
};

export const subtractDaysFromDate = (date, days) => {
	return subDays(date, days);
};

export const compareDates = (date1, date2) => {
	return {
		isDate1After: isAfter(date1, date2),
		isDate1Before: isBefore(date1, date2)
	};
};

// Some internal functions that shouldn't be exposed
const _internalFormatter = date => {
	return format(date, "dd/MM/yyyy HH:mm:ss");
};

const _internalParser = str => {
	return parseISO(str);
};
