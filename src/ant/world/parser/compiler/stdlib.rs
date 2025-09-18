pub const STDLIB: &str = r#"
	# THE ANTBYTE STANDARD LIBRARY

	set desc = "antbyte standard library";

	## And (2-4 parameters)

	fn and = (i0, i1) => out { out = -or(-i0, -i1); }
	fn and = (i0, i1, i2) => out { out = -or(-i0, -i1, -i2); }
	fn and = (i0, i1, i2, i3) => out { out = -or(-i0, -i1, -i2, -i3); }


	## Other Logic Gates

	fn xor = (a, b) => c { c = or(and(-a, b), and(a, -b)); }

	fn eq = (a, b) => c { c = or(and(a, b), and(-a, -b)); }

	fn mux = (s, a, b) => out { out = or(and(-s, a), and(s, b)); }

	fn mux = (s0, s1, a, b, c, d) => out {
		out = mux(s1, mux(s0, a, b), mux(s0, c, d));
	}

	## Addition & Subtraction

	fn add = (a, b) => (sum, cout) {
		sum = xor(a, b);
		cout = and(a, b);
	}

	fn add = (a, b, cin) => (sum, cout) {
		(sum0, cout0) = add(a, b);
		(sum,  cout1) = add(sum0, cin);
		cout = or(cout0, cout1);
	}


	## Copy: copies a single parameter to multiple assignees

	fn cpy = in => (o0) { o0 = in; }
	fn cpy = in => (o0, o1) { o0 = in; o1 = in; }
	fn cpy = in => (o0, o1, o2) { o0 = in; o1 = in; o2 = in; }
	fn cpy = in => (o0, o1, o2, o3) { o0 = in; o1 = in; o2 = in; o2 = in; }


	## Buffer: passes multiple parameters to multiple assignees

	fn buf = (i0) => (o0) { o0 = i0; }
	fn buf = (i0, i1) => (o0, o1) { o0 = i0; o1 = i1; }
	fn buf = (i0, i1, i2) => (o0, o1, o2) { o0 = i0; o1 = i1; o2 = i2; }
	fn buf = (i0, i1, i2, i3) => (o0, o1, o2, o3) {
		o0 = i0; o1 = i1; o2 = i2; o3 = i3;
	}


	## Invert: same as `buf()`, but with inverted parameter values

	fn inv = (i0) => (o0) { o0 = -i0; }
	fn inv = (i0, i1) => (o0, o1) { o0 = -i0; o1 = -i1; }
	fn inv = (i0, i1, i2) => (o0, o1, o2) { o0 = -i0; o1 = -i1; o2 = -i2; }
	fn inv = (i0, i1, i2, i3) => (o0, o1, o2, o3) {
		o0 = -i0; o1 = -i1; o2 = -i2; o3 = -i3;
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
"#;
