#!/usr/bin/env bash

should_exit=0
child_pid=""
gif_enabled=0

world_script="js/examples/random_world.mjs"
world_settings_normal="border=die;decay=128;height=128;width=216;speed=4;fps=12;dur=45"
world_settings_gif="border=die;decay=1024;size=384;speed=8;fps=24;dur=25"

for arg in "$@"; do
	case "$arg" in
		--gif)
			gif_enabled=1
			;;
		*)
			echo "Usage: $0 [--gif]" >&2
			exit 1
			;;
	esac
done

random_gif_name() {
	local suffix
	suffix=$(tr -dc 'a-z0-9' </dev/urandom | head -c 8)
	echo "generated/random_${suffix}.gif"
}

run_antbyte() {
	if (( gif_enabled )); then
		local gif_name
		gif_name=$(random_gif_name)
		echo "Creating GIF: $gif_name"
		antbyte --gif "$gif_name" "$world_script" -Tc "$world_settings_gif" &
	else
		antbyte "$world_script" -Tc "$world_settings_normal" &
	fi
	child_pid=$!
}

stop_child() {
	if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
		kill "$child_pid" 2>/dev/null
	fi
}

on_int() {
	should_exit=1
	stop_child
}

trap on_int INT
trap 'stop_child' EXIT

while (( !should_exit )); do
	skip_delay=0
	run_antbyte

	# Watch keyboard input while antbyte is running.
	while kill -0 "$child_pid" 2>/dev/null; do
		key=""
		IFS= read -rsn1 -t 0.1 key < /dev/tty
		read_status=$?

		if [[ $read_status -eq 0 && $key == $'\x04' ]]; then
			# Ctrl+D: stop current antbyte run and continue the loop immediately.
			skip_delay=1
			stop_child
			break
		fi

		if [[ $read_status -eq 1 ]]; then
			# Ctrl+D is reported as EOF by read in canonical terminal mode.
			skip_delay=1
			stop_child
			break
		fi
	done

	wait "$child_pid" 2>/dev/null
	child_pid=""

	if (( should_exit )); then
		break
	fi

	if (( skip_delay )); then
		continue
	fi

	sleep 2
done
