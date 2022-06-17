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

test: unit-test e2e-test

dev-deploy: contract ?= popskl.${owner}
dev-deploy: test	
	-near delete ${contract} ${owner}
	near create-account ${contract} --masterAccount ${owner}

	near deploy ${contract} \
		--wasmFile res/popskl.wasm \
		--initFunction new \
		--initArgs "{\"owner\": \"${owner}\"}"

build-scripts:
	cd scripts; \
		npm install; \
		npx tsc
