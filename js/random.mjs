/** @returns {number} */
export function random() {
	return Math.random()
}

/** @param {number} n @returns {number} */
export function randomInt(n) {
	return Math.floor(random() * n)
}

/** @param {number} probability @returns {boolean} */
export function randomChance(probability) {
	if (probability < 0 || probability > 1) throw new RangeError('Probability must be between 0 and 1')

	return random() < probability
}
