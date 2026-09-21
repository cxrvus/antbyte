SRC="src/parser/compiler/stdlib.rs"
DEST="../antbyte-examples/lib/std.ant"
cat $SRC | tail -n +2 | head -n -1 > $DEST
