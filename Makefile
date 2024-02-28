all:
	cargo build --release

install:
	install ./target/release/x-right-touch /usr/local/bin/
	chmod u+s /usr/local/bin/x-right-touch
