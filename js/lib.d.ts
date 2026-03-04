type World = {
	cfg: WorldConfig,
	ants: Behaviors,
}

type WorldConfig = {
	width: number
	height: number
	fps?: number
	speed?: number
	ticks?: number
	sleep?: number
	looping: boolean
	border_mode: "collide" | "despawn" | "cycle" | "wrap"
	starting_pos: "top_left" | "middle_left" | "center"
	color_mode: "binary" | "rgbi"
	noise_seed?: number
	hide_title: boolean
	description: string
}

type Behaviors = { [key: number]: Behavior }

type Behavior = {
	name: string
	logic: number[]
	inputs: string[]
	outputs: string[]
}
