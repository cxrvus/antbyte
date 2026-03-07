// @ts-check
/** @import * as AntByte from "../lib" AntByte */

import { writeFileSync } from 'fs'

import { ant, run } from "../lib.mjs"
import { size, bits, byte, newWorld, peripherals} from "../util.mjs"

/** @returns {number} */
function random() {
	return Math.random()
}

/** @param {number} n @returns {number} */
function randomInt(n) {
	return Math.floor(random() * n)
}

/** @param {number} probability @returns {boolean} */
function randomChance(probability) {
	if (probability < 0 || probability > 1) throw new RangeError('Probability must be between 0 and 1')

	return random() < probability
}

/** @returns {AntByte.World} */
function generateWorld() {
	const world = newWorld()

	const antCount = randomInt(16) + 1

	for (let i = 0; i <= antCount; i++) {
		world.ants[i] = generateAnt(i.toString())
	}

	return world
}

/** @param {string} name @returns {AntByte.Behavior} */
function generateAnt(name) {
	// manual tweaking...
	const mandatoryOutputs = ['A0', 'A1', 'A2', 'A3']
	const blockedOutputs = [...mandatoryOutputs, ['A4', 'A5', 'A6', 'A7']]
	const filteredOutputs = peripherals.output.filter(p => !blockedOutputs.includes(p))
	//

	const inputs = getSubset(peripherals.input, randomInt(4) + 4);
	const outputs = getSubset(filteredOutputs, randomInt(12) + 4);

	// more manual tweaking...
	//  increase chance of spawning
	outputs.concat(mandatoryOutputs)

	// decrease chance of dying
	if (outputs.includes('AX') && (randomChance(0.8))) {
		outputs.splice(outputs.indexOf('AX'), 1)
	}
	//

	const inputCount = inputs.length;
	const outputCount = outputs.length;

	const valueCount = 2 ** inputCount
	const maxValue = 2 ** outputCount + 1

	const logic = []

	for (let i = 0; i < valueCount; i++) logic.push(randomInt(maxValue))

	return { name, outputs, inputs, logic }
}

/** @param {string[]} superSet @param {number} amount @returns {string[]} */
function getSubset(superSet, amount) {
	const set = [...superSet]
	/** @type {string[]} */
	const subset = [];
	let setSize = set.length;

	while (subset.length < amount) {
		const index = randomInt(setSize);
		subset.push(set[index]);
		set.splice(index, 1);
		setSize--;
	}

	return subset
}

const world = generateWorld()

world.cfg = {
	// @ts-ignore
	border_mode: "despawn",
	...size(64),
}

const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '-')
const dirname = import.meta.dirname;
writeFileSync(`${dirname}/../../tmp/random_world-${timestamp}.ant.json`, JSON.stringify(world), 'utf-8')

run(world)
