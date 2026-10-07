pub const STDLIB: &str = r#"
# THE ANTBYTE STANDARD LIBRARY

no_std;
set desc = "antbyte standard library";


## And (2-4 parameters)

def and(i0, i1) => (out) { out = -or(-i0, -i1); }
def and(i0, i1, i2) => (out) { out = -or(-i0, -i1, -i2); }
def and(i0, i1, i2, i3) => (out) { out = -or(-i0, -i1, -i2, -i3); }


## Other Logic Gates

def xor(a, b) => (c) { c = or(and(-a, +b), and(+a, -b)); }

def eq(a, b) => (out) { out = or(and(a, b), -or(a, b)); }
def eq(a, b, c) => (out) { out = or(and(a, b, c), -or(a, b, c)); }
def eq(a, b, c, d) => (out) { out = or(and(a, b, c, d), -or(a, b, c, d)); }

def imply(a, b) => (c) { c = or(-a, b); }


## Multiplexer

def mux(s, a, b) => (out) { out = or(and(-s, a), and(s, b)); }

def mux(s1, s0, a, b, c, d) => (out) {
	out = mux(s1, mux(s0, a, b), mux(s0, c, d));
}

def mux(s2, s1, s0, o0, o1, o2, o3, o4, o5, o6, o7) => (out) {
	out = mux(s2,
		mux(s1, mux(s0, o0, o1), mux(s0, o2, o3)),
		mux(s1, mux(s0, o4, o5), mux(s0, o6, o7)),
	);
}

def mux(
		s3, s2, s1, s0,
		o00, o01, o02, o03, o04, o05, o06, o07,
		o08, o09, o10, o11, o12, o13, o14, o15
	) => (out) {
	out = mux(s3,
		mux(s2,
			mux(s1, mux(s0, o00, o01), mux(s0, o02, o03)),
			mux(s1, mux(s0, o04, o05), mux(s0, o06, o07)),
		),
		mux(s2,
			mux(s1, mux(s0, o08, o09), mux(s0, o10, o11)),
			mux(s1, mux(s0, o12, o13), mux(s0, o14, o15)),
		),
	);
}


## Addition

def add(a, b) => (cout, sum) {
	sum = xor(a, b);
	cout = and(a, b);
}

def add(a, b, cin) => (cout, sum) {
	(cout0, sum) = add(a, b);
	(cout1, sum) = add(sum, cin);
	cout = or(cout0, cout1);
}

def add(a1, a0, b1, b0, cin) => (cout, sum1, sum0) {
	cout = cin;
	(cout, sum0) = add(a0, b0, cout);
	(cout, sum1) = add(a1, b1, cout);
}

def add(a1, a0, b1, b0) => (cout, sum1, sum0) {
	(cout, sum1, sum0) = add(a1, a0, b1, b0, 0);
}

def add(a2, a1, a0, b2, b1, b0, cin) => (cout, sum2, sum1, sum0) {
	cout = cin;
	(cout, sum0) = add(a0, b0, cout);
	(cout, sum1) = add(a1, b1, cout);
	(cout, sum2) = add(a2, b2, cout);
}

def add(a2, a1, a0, b2, b1, b0) => (cout, sum2, sum1, sum0) {
	(cout, sum2, sum1, sum0) = add(a2, a1, a0, b2, b1, b0, 0);
}

def add(a3, a2, a1, a0, b3, b2, b1, b0, cin) => (cout, sum3, sum2, sum1, sum0) {
	cout = cin;
	(cout, sum0) = add(a0, b0, cout);
	(cout, sum1) = add(a1, b1, cout);
	(cout, sum2) = add(a2, b2, cout);
	(cout, sum3) = add(a3, b3, cout);
}

def add(a3, a2, a1, a0, b3, b2, b1, b0) => (cout, sum3, sum2, sum1, sum0) {
	(cout, sum3, sum2, sum1, sum0) = add(a3, a2, a1, a0, b3, b2, b1, b0, 0);
}


## Copy: copies a single parameter to multiple assignees

def cpy(in) => (o0) { o0 = in; }
def cpy(in) => (o0, o1) { o0 = in; o1 = in; }
def cpy(in) => (o0, o1, o2) { o0 = in; o1 = in; o2 = in; }
def cpy(in) => (o0, o1, o2, o3) { o0 = in; o1 = in; o2 = in; o3 = in; }


## Forward: passes multiple parameters to multiple assignees

def fwd(i0) => (o0) { o0 = i0; }
def fwd(i0, i1) => (o0, o1) { o0 = i0; o1 = i1; }
def fwd(i0, i1, i2) => (o0, o1, o2) { o0 = i0; o1 = i1; o2 = i2; }
def fwd(i0, i1, i2, i3) => (o0, o1, o2, o3) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3;
}


## Forward (AND)

def fwd_and(e, i1, i0) => (o1, o0) {
	o1 = and(e, i1); o0 = and(e, i0);
}

def fwd_and(e, i2, i1, i0) => (o2, o1, o0) {
	o2 = and(e, i2); o1 = and(e, i1); o0 = and(e, i0);
}

def fwd_and(e, i3, i2, i1, i0) => (o3, o2, o1, o0) {
	o3 = and(e, i3); o2 = and(e, i2); o1 = and(e, i1);
	o0 = and(e, i0);
}

def fwd_and(e, i4, i3, i2, i1, i0) => (o4, o3, o2, o1, o0) {
	o4 = and(e, i4); o3 = and(e, i3); o2 = and(e, i2);
	o1 = and(e, i1); o0 = and(e, i0);
}

def fwd_and(e, i5, i4, i3, i2, i1, i0) => (o5, o4, o3, o2, o1, o0) {
	o5 = and(e, i5); o4 = and(e, i4); o3 = and(e, i3);
	o2 = and(e, i2); o1 = and(e, i1); o0 = and(e, i0);
}


## Forward (OR)

def fwd_or(e, i1, i0) => (o1, o0) {
	o1 = or(e, i1); o0 = or(e, i0);
}

def fwd_or(e, i2, i1, i0) => (o2, o1, o0) {
	o2 = or(e, i2); o1 = or(e, i1); o0 = or(e, i0);
}

def fwd_or(e, i3, i2, i1, i0) => (o3, o2, o1, o0) {
	o3 = or(e, i3); o2 = or(e, i2); o1 = or(e, i1);
	o0 = or(e, i0);
}


## Forward (XOR)

def fwd_xor(e, i1, i0) => (o1, o0) {
	o1 = xor(e, i1); o0 = xor(e, i0);
}

def fwd_xor(e, i2, i1, i0) => (o2, o1, o0) {
	o2 = xor(e, i2); o1 = xor(e, i1); o0 = xor(e, i0);
}

def fwd_xor(e, i3, i2, i1, i0) => (o3, o2, o1, o0) {
	o3 = xor(e, i3); o2 = xor(e, i2); o1 = xor(e, i1);
	o0 = xor(e, i0);
}


## 5-8 Parameter Versions of Previous Functions

def and(i0, i1, i2, i3, i4) => (out) { out = -or(-i0, -i1, -i2, -i3, -i4); }
def and(i0, i1, i2, i3, i4, i5) => (out) { out = -or(-i0, -i1, -i2, -i3, -i4, -i5); }
def and(i0, i1, i2, i3, i4, i5, i6) => (out) { out = -or(-i0, -i1, -i2, -i3, -i4, -i5, -i6); }
def and(i0, i1, i2, i3, i4, i5, i6, i7) => (out) { out = -or(-i0, -i1, -i2, -i3, -i4, -i5, -i6, -i7); }

def fwd(i0, i1, i2, i3, i4) => (o0, o1, o2, o3, o4) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3; o4 = i4;
}
def fwd(i0, i1, i2, i3, i4, i5) => (o0, o1, o2, o3, o4, o5) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3; o4 = i4; o5 = i5;
}
def fwd(i0, i1, i2, i3, i4, i5, i6) => (o0, o1, o2, o3, o4, o5, o6) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3; o4 = i4; o5 = i5; o6 = i6;
}
def fwd(i0, i1, i2, i3, i4, i5, i6, i7) => (o0, o1, o2, o3, o4, o5, o6, o7) {
	o0 = i0; o1 = i1; o2 = i2; o3 = i3; o4 = i4; o5 = i5; o6 = i6; o7 = i7;
}

def cpy(in) => (o0, o1, o2, o3, o4) {
	o0 = in; o1 = in; o2 = in; o3 = in; o4 = in;
}
def cpy(in) => (o0, o1, o2, o3, o4, o5) {
	o0 = in; o1 = in; o2 = in; o3 = in; o4 = in; o5 = in;
}
def cpy(in) => (o0, o1, o2, o3, o4, o5, o6) {
	o0 = in; o1 = in; o2 = in; o3 = in; o4 = in; o5 = in; o6 = in;
}
def cpy(in) => (o0, o1, o2, o3, o4, o5, o6, o7) {
	o0 = in; o1 = in; o2 = in; o3 = in; o4 = in; o5 = in; o6 = in; o7 = in;
}


## Parallel Equals
def p_eq(a1, a0, b1, b0) => (out) { out = and(eq(a1, b1), eq(a0, b0),); }

def p_eq(a2, a1, a0, b2, b1, b0) => (out) {
	out = and(eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

def p_eq(a3, a2, a1, a0, b3, b2, b1, b0) => (out) {
	out = and(eq(a3, b3), eq(a2, b2), eq(a1, b1), eq(a0, b0),);
}

def p_eq(a4, a3, a2, a1, a0, b4, b3, b2, b1, b0) => (out) {
	out = and(
		eq(a4, b4), eq(a3, b3), eq(a2, b2),
		eq(a1, b1), eq(a0, b0),
	);
}

def p_eq(a5, a4, a3, a2, a1, a0, b5, b4, b3, b2, b1, b0) => (out) {
	out = and(
		eq(a5, b5), eq(a4, b4), eq(a3, b3),
		eq(a2, b2), eq(a1, b1), eq(a0, b0),
	);
}


## Decoder

def dec(i1, i0) => (o0, o1, o2, o3) {
	o0 = p_eq(i1, i0, 0, 0);
	o1 = p_eq(i1, i0, 0, 1);
	o2 = p_eq(i1, i0, 1, 0);
	o3 = p_eq(i1, i0, 1, 1);
}

def dec(i2, i1, i0) => (o0, o1, o2, o3, o4, o5, o6, o7) {
	o0 = p_eq(i2, i1, i0, 0, 0, 0); o1 = p_eq(i2, i1, i0, 0, 0, 1);
	o2 = p_eq(i2, i1, i0, 0, 1, 0); o3 = p_eq(i2, i1, i0, 0, 1, 1);

	o4 = p_eq(i2, i1, i0, 1, 0, 0); o5 = p_eq(i2, i1, i0, 1, 0, 1);
	o6 = p_eq(i2, i1, i0, 1, 1, 0); o7 = p_eq(i2, i1, i0, 1, 1, 1);
}

def dec(i3, i2, i1, i0) => (o00, o01, o02, o03, o04, o05, o06, o07, o08, o09, o10, o11, o12, o13, o14, o15) {
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

def enc(i0, i1, i2, i3) => (o1, o0) {
	u0 = i0;
	u1 = and(i1, -i0);
	u2 = and(i2, -i1, -i0);
	u3 = and(i3, -i2, -i1, -i0);

	o0 = or(u1, u3);
	o1 = or(u2, u3);
}

def enc(i0, i1, i2, i3, i4, i5, i6, i7) => (o2, o1, o0) {
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

def hw(i3, i2, i1, i0) => (hw2, hw1, hw0) {
	(cout_a0, a0) = add(i1, i0);
	(cout_a1, a1) = add(i3, i2);

	(hw2, hw1, hw0) = add(cout_a0, a0, cout_a1, a1);
}

def hw(i7, i6, i5, i4, i3, i2, i1, i0) => (hw3, hw2, hw1, hw0) {
	(hw02, hw01, hw00) = hw(i3, i2, i1, i0);
	(hw12, hw11, hw10) = hw(i7, i6, i5, i4);

	(cout, hw1, hw0) = add(hw01, hw00, hw11, hw10);
	(hw3, hw2) = add(cout, hw02, hw12);
}


## Hamming weight = 1

def one(a, b) => (out) { out = xor(a,b); }

def one(a, b, c) => (out) {
	out = or(
		and(+a, -b, -c),
		and(-a, +b, -c),
		and(-a, -b, +c),
	);
}

def one(a, b, c, d) => (out) {
	out = or(
		and(+a, -b, -c, -d),
		and(-a, +b, -c, -d),
		and(-a, -b, +c, -d),
		and(-a, -b, -c, +d),
	);
}


## Counter

def on(t1, t0) => (out) {
	out = p_eq(CTR_1, CTR_0, t1, t0);
}

def on(t2, t1, t0) => (out) {
	out = p_eq(CTR_2, CTR_1, CTR_0, t2, t1, t0);
}

def on(t3, t2, t1, t0) => (out) {
	out = p_eq(CTR_3, CTR_2, CTR_1, CTR_0, t3, t2, t1, t0);
}

def on(t5, t4, t3, t2, t1, t0) => (out) {
	out = p_eq(CTR_5, CTR_4, CTR_3, CTR_2, CTR_1, CTR_0, 
					t5, t4, t3, t2, t1, t0);
}


## Rotation

def rot_64(l, r) => (r0, r1) 	{ ROT_1 = or(r, l); ROT_0 = l; 			(r0, r1) = fwd(ROT_0, ROT_1); }
def rot_32(l, r) => (r0, r1, r2) 	{ ROT_2 = or(r, l); ROT_1 = l; ROT_0 = l; 	(r0, r1, r2) = fwd(ROT_0, ROT_1, ROT_2); }

def rot_64(l, r) { (_, _) = rot_64(l, r); }
def rot_32(l, r) { (_, _, _) = rot_32(l, r); }

"#;
