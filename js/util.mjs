/** @param {...boolean} params  */
export function toByte(...params) {
	let result = 0
	for (let i = 0; i < params.length; i++) {
		if (params[i]) result |= (1 << (params.length - 1 - i))
	}
	return result
}

// todo: wrapper functions for 1-8 bits

/** @param {number} count @param {number} value @returns {boolean[]}  */
export function toBits(count, value) {
	const result = []
	for (let i = count - 1; i >= 0; i--) {
		result[count - 1 - i] = (value & (1 << i)) !== 0
	}
	return result
}
