// @ts-check

/** @param {...boolean} params  */
export function byte(...params) {
	let result = 0
	for (let i = 0; i < params.length; i++) {
		if (params[i]) result |= (1 << (params.length - 1 - i))
	}
	return result
}

/** @param {number} count @param {number} value @returns {boolean[]}  */
export function bits(count, value) {
	if (value > (2 ** count - 1)) {
		throw new RangeError(`the number ${value} cannot be displayed using ${count} bytes`)
	} else if (value < 0) {
		throw new RangeError(`cannot display negative number ${value}`)
	} 

	const result = []
	for (let i = count - 1; i >= 0; i--) {
		result[count - 1 - i] = (value & (1 << i)) !== 0
	}
	return result
}


// config wrappers

/** @param {number} value */
export const size = value => ({ height: value, width: value})


// wrapper functions for 1-8 bits

/** @param {number} value @returns {boolean[]}  */
export const bits_2 = value => bits(2, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_3 = value => bits(3, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_4 = value => bits(4, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_5 = value => bits(5, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_6 = value => bits(6, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_7 = value => bits(7, value);

/** @param {number} value @returns {boolean[]}  */
export const bits_8 = value => bits(8, value);


// other wrappers

/** @returns {import('./lib').World} */
export const newWorld = () => ({ cfg: {}, ants: {} })

export const $0 = false;
export const $1 = true;

/** @param {string} msg */
export const err = msg => console.error(msg)

// from stdlib.ant

/** @param {boolean[]} params @returns {boolean} */
export const and = (...params) => params.every(x => x)

/** @param {boolean[]} params @returns {boolean} */
export const or = (...params) => !params.every(x => !x)

/** @param {boolean[]} params @returns {boolean[]} */
export const inv = (...params) => params.map(x => !x)


// manually created event spec

export const events = {
	input: [
		"AC",
		"C0", "C1", "C2", "C3",
		"CN0", "CN1", "CN2", "CN3",
		"T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7",
		"TT0", "TT1", "TT2", "TT3", "TT4", "TT5", "TT6", "TT7",
		"R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7",
		"RR0", "RR1", "RR2", "RR3", "RR4", "RR5", "RR6", "RR7",
		"M0", "M1", "M2", "M3", "M4", "M5", "M6", "M7",
	],

	output: [
		"CC",
		"C0", "C1", "C2", "C3",
		"M0", "M1", "M2", "M3", "M4", "M5", "M6", "M7",
		"D0", "D1", "D2", "DX",
		"A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7",
		"AK", "AX",
	],
}
