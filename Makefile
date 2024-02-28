all:
	cargo build --release

install:
	sudo install ./target/release/x-right-touch /usr/local/bin/
	sudo chmod u+s /usr/local/bin/x-right-touch
