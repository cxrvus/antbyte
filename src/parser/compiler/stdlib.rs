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


## Parallel Equals
fn p_eq = (a1, a0, b1, b0) => out { out = and(eq(a1, b1), eq(a0, b0),); }

fn p_eq = (a2, a1, a0, b2, b1, b0) => out {
	out = and(eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

fn p_eq = (a3, a2, a1, a0, b3, b2, b1, b0) => out {
	out = and(eq(a3, b3), eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

fn p_eq = (a4, a3, a2, a1, a0, b4, b3, b2, b1, b0) => out {
	out = and(
		eq(a4, b4), eq(a3, b3), eq(a2, b2),
		eq(a1, b1), eq(a0, b0),
	);
}

fn p_eq = (a5, a4, a3, a2, a1, a0, b5, b4, b3, b2, b1, b0) => out {
	out = and(
		eq(a5, b5), eq(a4, b4), eq(a3, b3),
		eq(a2, b2), eq(a1, b1), eq(a0, b0),
	);
}


## Enabler

fn enb = (e, i1, i0) => (o1, o0) {
	o1 = and(e, i1); o0 = and(e, i0);
}

fn enb = (e, i2, i1, i0) => (o2, o1, o0) {
	o2 = and(e, i2); o1 = and(e, i1); o0 = and(e, i0);
}

fn enb = (e, i3, i2, i1, i0) => (o3, o2, o1, o0) {
	o3 = and(e, i3); o2 = and(e, i2); o1 = and(e, i1);
	o0 = and(e, i0);
}

fn enb = (e, i4, i3, i2, i1, i0) => (o4, o3, o2, o1, o0) {
	o4 = and(e, i4); o3 = and(e, i3); o2 = and(e, i2);
	o1 = and(e, i1); o0 = and(e, i0);
}

fn enb = (e, i5, i4, i3, i2, i1, i0) => (o5, o4, o3, o2, o1, o0) {
	o5 = and(e, i5); o4 = and(e, i4); o3 = and(e, i3);
	o2 = and(e, i2); o1 = and(e, i1); o0 = and(e, i0);
}


## Decoder

fn dec = (i1, i0) => (o0, o1, o2, o3) {
	o0 = p_eq(i1, i0, 0, 0);
	o1 = p_eq(i1, i0, 0, 1);
	o2 = p_eq(i1, i0, 1, 0);
	o3 = p_eq(i1, i0, 1, 1);
}

fn dec = (i2, i1, i0) => (o0, o1, o2, o3, o4, o5, o6, o7) {
	o0 = p_eq(i2, i1, i0, 0, 0, 0); o1 = p_eq(i2, i1, i0, 0, 0, 1);
	o2 = p_eq(i2, i1, i0, 0, 1, 0); o3 = p_eq(i2, i1, i0, 0, 1, 1);

	o4 = p_eq(i2, i1, i0, 1, 0, 0); o5 = p_eq(i2, i1, i0, 1, 0, 1);
	o6 = p_eq(i2, i1, i0, 1, 1, 0); o7 = p_eq(i2, i1, i0, 1, 1, 1);
}

fn dec = (i3, i2, i1, i0) => (o00, o01, o02, o03, o04, o05, o06, o07, o08, o09, o10, o11, o12, o13, o14, o15) {
	o00 = p_eq(i3, i2, i1, i0, 0, 0, 0, 0); o01 = p_eq(i3, i2, i1, i0, 0, 0, 0, 1);
	o02 = p_eq(i3, i2, i1, i0, 0, 0, 1, 0); o03 = p_eq(i3, i2, i1, i0, 0, 0, 1, 1);

	o04 = p_eq(i3, i2, i1, i0, 0, 1, 0, 0); o05 = p_eq(i3, i2, i1, i0, 0, 1, 0, 1);
	o06 = p_eq(i3, i2, i1, i0, 0, 1, 1, 0); o07 = p_eq(i3, i2, i1, i0, 0, 1, 1, 1);

	o08 = p_eq(i3, i2, i1, i0, 1, 0, 0, 0); o09 = p_eq(i3, i2, i1, i0, 1, 0, 0, 1);
	o10 = p_eq(i3, i2, i1, i0, 1, 0, 1, 0); o11 = p_eq(i3, i2, i1, i0, 1, 0, 1, 1);

	o12 = p_eq(i3, i2, i1, i0, 1, 1, 0, 0); o13 = p_eq(i3, i2, i1, i0, 1, 1, 0, 1);
	o14 = p_eq(i3, i2, i1, i0, 1, 1, 1, 0); o15 = p_eq(i3, i2, i1, i0, 1, 1, 1, 1);
}


## Encoder

fn enc = (i0, i1, i2, i3) => (o1, o0) {
	u0 = i0;
	u1 = and(i1, -i0);
	u2 = and(i2, -i1, -i0);
	u3 = and(i3, -i2, -i1, -i0);

	o0 = or(u1, u3);
	o1 = or(u2, u3);
}

fn enc = (i0, i1, i2, i3, i4, i5, i6, i7) => (o2, o1, o0) {
	u0 = i0;
	u1 = and(i1, -i0);
	u2 = and(i2, -i1, -i0);
	u3 = and(i3, -i2, -i1, -i0);
	u4 = and(i4, -i3, -i2, -i1, -i0);
	u5 = and(i5, -i4, -i3, -i2, -i1, -i0);
	u6 = and(i6, -i5, -i4, -i3, -i2, -i1, -i0);
	u7 = and(i7, -i6, -i5, -i4, -i3, -i2, -i1, -i0);

	o0 = or(u1, u3, u5, u7);
	o1 = or(u2, u3, u6, u7);
	o2 = or(u4, u5, u6, u7);
}


## Hamming weight

fn hw = (i3, i2, i1, i0) => (hw2, hw1, hw0) {
	(cout_a0, a0) = add(i1, i0);
	(cout_a1, a1) = add(i3, i2);

	(hw2, hw1, hw0) = add(cout_a0, a0, cout_a1, a1);
}

fn hw = (i7, i6, i5, i4, i3, i2, i1, i0) => (hw3, hw2, hw1, hw0) {
	(hw02, hw01, hw00) = hw(i3, i2, i1, i0);
	(hw12, hw11, hw10) = hw(i7, i6, i5, i4);

	(cout, hw1, hw0) = add(hw01, hw00, hw11, hw10);
	(hw3, hw2) = add(cout, hw02, hw12);
}


## Hamming weight = 1

fn one = (a, b) => out { out = xor(a,b); }

fn one = (a, b, c) => out {
	out = or(
		and(+a, -b, -c),
		and(-a, +b, -c),
		and(-a, -b, +c),
	);
}

fn one = (a, b, c, d) => out {
	out = or(
		and(+a, -b, -c, -d),
		and(-a, +b, -c, -d),
		and(-a, -b, +c, -d),
		and(-a, -b, -c, +d),
	);
}


## Clock

fn on = (t1, t0) => out {
	out = p_eq(T1, T0, t1, t0);
}

fn on = (t2, t1, t0) => out {
	out = p_eq(T2, T1, T0, t2, t1, t0);
}

fn on = (t3, t2, t1, t0) => out {
	out = p_eq(T3, T2, T1, T0, t3, t2, t1, t0);
}

fn on = (t5, t4, t3, t2, t1, t0) => out {
	out = p_eq(	T5, T4, T3, T2, T1, T0, 
					t5, t4, t3, t2, t1, t0);
}
"#;
