.PHONY: build test run-services stop-services clean

build:
	cargo build --manifest-path orchestrator/Cargo.toml
	cd onchain && anchor build

test:
	cargo test --manifest-path orchestrator/Cargo.toml
	cd onchain && anchor test

run-services:
	docker-compose up -d

stop-services:
	docker-compose down

clean:
	cargo clean --manifest-path orchestrator/Cargo.toml
	cd onchain && anchor clean
	rm -rf sdk-client/dist
