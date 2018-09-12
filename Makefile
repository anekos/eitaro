
release:
	RUSTC_WRAPPER=`which sccache` cargo build --release
