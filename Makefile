
release:
	RUSTC_WRAPPER=`which sccache` cargo build --release

webextension:
	- rm eitaro.zip
	(cd webext ; zip -r ../eitaro.zip .)

test:
	cargo test
	@echo ''
	bash test.sh
