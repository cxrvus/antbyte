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

export const $0 = false;
export const $1 = true;

/** @param {string} msg */
export const err = msg => console.error(msg)
