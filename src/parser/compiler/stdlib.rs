pub const STDLIB: &str = r#"
# THE ANTBYTE STANDARD LIBRARY

no_std;
set desc = "antbyte standard library";


## And (2-4 parameters)

fn and = (i0, i1) => out { out = -or(-i0, -i1); }
fn and = (i0, i1, i2) => out { out = -or(-i0, -i1, -i2); }
fn and = (i0, i1, i2, i3) => out { out = -or(-i0, -i1, -i2, -i3); }


## Other Logic Gates

fn xor = (a, b) => c { c = or(and(-a, +b), and(+a, -b)); }

fn eq  = (a, b) => out { out = or(and(a, b), -or(a, b)); }
fn eq  = (a, b, c) => out { out = or(and(a, b, c), -or(a, b, c)); }
fn eq  = (a, b, c, d) => out { out = or(and(a, b, c, d), -or(a, b, c, d)); }

fn imply = (a, b) => c { c = or(-a, b); }

fn mux = (s, a, b) => out { out = or(and(-s, a), and(s, b)); }

fn mux = (s0, s1, a, b, c, d) => out {
	out = mux(s1, mux(s0, a, b), mux(s0, c, d));
}


## Addition & Subtraction

fn add = (a, b) => (cout, sum) {
	sum = xor(a, b);
	cout = and(a, b);
}

fn add = (a, b, cin) => (cout, sum) {
	(cout0, sum0) = add(a, b);
	(cout1, sum) = add(sum0, cin);
	cout = or(cout0, cout1);
}

fn add = (a1, a0, b1, b0) => (cout, sum1, sum0) {
	(cout0, sum0) = add(a0, b0);
	(cout, sum1) = add(a1, b1, cout0);
}

fn add = (a1, a0, b1, b0, cin) => (cout, sum1, sum0) {
	(cout0, sum0) = add(a0, b0, cin);
	(cout, sum1) = add(a1, b1, cout0);
}


## Copy: copies a single parameter to multiple assignees

fn cpy = in => (o0) { o0 = in; }
fn cpy = in => (o0, o1) { o0 = in; o1 = in; }
fn cpy = in => (o0, o1, o2) { o0 = in; o1 = in; o2 = in; }
fn cpy = in => (o0, o1, o2, o3) { o0 = in; o1 = in; o2 = in; o3 = in; }


## Buffer: passes multiple parameters to multiple assignees

fn buf = (i0) => (o0) { o0 = i0; }
fn buf = (i0, i1) => (o0, o1) { o0 = i0; o1 = i1; }
fn buf = (i0, i1, i2) => (o0, o1, o2) { o0 = i0; o1 = i1; o2 = i2; }
fn buf = (i0, i1, i2, i3) => (o0, o1, o2, o3) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3;
}


## And (5-8 parameters)

fn and = (i0, i1, i2, i3, i4) => out {
	out = -or(-i0, -i1, -i2, -i3, -i4);
}

fn and = (i0, i1, i2, i3, i4, i5) => out {
	out = -or(-i0, -i1, -i2, -i3, -i4, -i5);
}

fn and = (i0, i1, i2, i3, i4, i5, i6) => out {
	out = -or(-i0, -i1, -i2, -i3, -i4, -i5, -i6);
}

fn and = (i0, i1, i2, i3, i4, i5, i6, i7) => out {
	out = -or(-i0, -i1, -i2, -i3, -i4, -i5, -i6, -i7);
}


## Pairwise Equals
fn pair_eq = (a1, a0, b1, b0) => out { out = and(eq(a1, b1), eq(a0, b0),); }

fn pair_eq = (a2, a1, a0, b2, b1, b0) => out {
	out = and(eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

fn pair_eq = (a3, a2, a1, a0, b3, b2, b1, b0) => out {
	out = and(eq(a3, b3), eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

fn pair_eq = (a4, a3, a2, a1, a0, b4, b3, b2, b1, b0) => out {
	out = and(
		eq(a4, b4), eq(a3, b3), eq(a2, b2),
		eq(a1, b1), eq(a0, b0),
	);
}

fn pair_eq = (a5, a4, a3, a2, a1, a0, b5, b4, b3, b2, b1, b0) => out {
	out = and(
		eq(a5, b5), eq(a4, b4), eq(a3, b3),
		eq(a2, b2), eq(a1, b1), eq(a0, b0),
	);
}


## Decoder

fn dec = (i1, i0) => (o0, o1, o2, o3) {
	o0 = pair_eq(i1, i0, 0, 0);
	o1 = pair_eq(i1, i0, 0, 1);
	o2 = pair_eq(i1, i0, 1, 0);
	o3 = pair_eq(i1, i0, 1, 1);
}

fn dec = (i2, i1, i0) => (o0, o1, o2, o3, o4, o5, o6, o7) {
	o0 = pair_eq(i2, i1, i0, 0, 0, 0);
	o1 = pair_eq(i2, i1, i0, 0, 0, 1);
	o2 = pair_eq(i2, i1, i0, 0, 1, 0);
	o3 = pair_eq(i2, i1, i0, 0, 1, 1);

	o4 = pair_eq(i2, i1, i0, 1, 0, 0);
	o5 = pair_eq(i2, i1, i0, 1, 0, 1);
	o6 = pair_eq(i2, i1, i0, 1, 1, 0);
	o7 = pair_eq(i2, i1, i0, 1, 1, 1);
}

fn dec = (i3, i2, i1, i0) => (o00, o01, o02, o03, o04, o05, o06, o07, o08, o09, o10, o11, o12, o13, o14, o15) {
	o00 = pair_eq(i3, i2, i1, i0, 0, 0, 0, 0);
	o01 = pair_eq(i3, i2, i1, i0, 0, 0, 0, 1);
	o02 = pair_eq(i3, i2, i1, i0, 0, 0, 1, 0);
	o03 = pair_eq(i3, i2, i1, i0, 0, 0, 1, 1);

	o04 = pair_eq(i3, i2, i1, i0, 0, 1, 0, 0);
	o05 = pair_eq(i3, i2, i1, i0, 0, 1, 0, 1);
	o06 = pair_eq(i3, i2, i1, i0, 0, 1, 1, 0);
	o07 = pair_eq(i3, i2, i1, i0, 0, 1, 1, 1);

	o08 = pair_eq(i3, i2, i1, i0, 1, 0, 0, 0);
	o09 = pair_eq(i3, i2, i1, i0, 1, 0, 0, 1);
	o10 = pair_eq(i3, i2, i1, i0, 1, 0, 1, 0);
	o11 = pair_eq(i3, i2, i1, i0, 1, 0, 1, 1);

	o12 = pair_eq(i3, i2, i1, i0, 1, 1, 0, 0);
	o13 = pair_eq(i3, i2, i1, i0, 1, 1, 0, 1);
	o14 = pair_eq(i3, i2, i1, i0, 1, 1, 1, 0);
	o15 = pair_eq(i3, i2, i1, i0, 1, 1, 1, 1);
}
"#;
