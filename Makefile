deps:
	cd ts && npm install

all:
	cd ts && npm run-script build
	cd rs && cargo build