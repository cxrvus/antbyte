#!/bin/bash

# idea: turn this script into a sub-command

# mirror_stdlib.sh
# Copies the stdlib content from stdlib.rs to std.ant
# Strips away the Rust wrapper and extracts only the content of const STDLIB

set -e  # Exit on any error

# Define file paths
STDLIB_RS="src/ant/world/parser/compiler/stdlib.rs"
STD_ANT="std.ant"

# Check if the source file exists
if [[ ! -f "$STDLIB_RS" ]]; then
    echo "Error: Source file $STDLIB_RS not found!"
    exit 1
fi

# Extract the content between r#" and "# from the STDLIB constant
# This uses sed to:
# 1. Find the line with 'pub const STDLIB: &str = r#"'
# 2. Print everything after that line until we find '"#;'
# 3. Remove the first and last lines (the Rust wrapper)

echo "Extracting stdlib content from $STDLIB_RS..."

# Use awk for more precise extraction
awk '
    /^pub const STDLIB: &str = r#"/ { 
        in_string = 1
        next  # Skip the line with the opening
    }
    /^"#;/ && in_string {
        in_string = 0
        exit  # Stop processing
    }
    in_string {
        print $0
    }
' "$STDLIB_RS" > "$STD_ANT"

# Check if extraction was successful
if [[ ! -s "$STD_ANT" ]]; then
    echo "Error: Failed to extract content or content is empty!"
    exit 1
fi

echo "Successfully mirrored stdlib content to $STD_ANT"

# Show a summary of what was copied
LINES=$(wc -l < "$STD_ANT")
echo "Copied $LINES lines to $STD_ANT"