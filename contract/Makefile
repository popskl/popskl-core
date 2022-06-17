build:
	RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/*.wasm ./res/

clean:
	rm -rf target res

unit-test:
	cargo test --lib -- --nocapture

e2e-test: build
	cargo test --test '*'

dev-deploy: contract ?= popskl.${account}
dev-deploy: unit-test build	
	near delete ${contract} ${account}
	near create-account ${contract} --masterAccount ${account}

	near deploy ${contract} \
		--wasmFile res/popskl.wasm \
		--initFunction new \
		--initArgs '{}'

build-scripts:
	cd scripts; \
		npm install; \
		npx tsc
