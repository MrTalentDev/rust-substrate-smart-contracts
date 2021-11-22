SHELL=/bin/sh

mkfile_path = $(abspath $(lastword $(MAKEFILE_LIST)))
current_dir = $(patsubst %/,%,$(dir $(mkfile_path)))
contracts_ci_linux_tag = 3582091f-20211118
contracts_ci_linux_image = paritytech/contracts-ci-linux
id_u = $(shell id -u)
id_g = $(shell id -g)

define run_ci_container =
	docker run --rm -it --name $(1) --user $(id_u):$(id_g) -w /work_path/$(2) -v $(current_dir):/work_path -v $(current_dir)/cache/:/cache/ -e CARGO_HOME=/cache/cargo/ -e SCCACHE_DIR=/cache/sccache/ --net substrate $(3) $(contracts_ci_linux_image):$(contracts_ci_linux_tag) $(4)
endef

create-network:
	docker network create --driver bridge substrate

new-contract:
ifdef contract_name
	$(call run_ci_container,create-contract,,,cargo contract new $(contract_name))
else
	$(error No contract_name argument provided)
endif

node-run:
	$(call run_ci_container,substrate-contracts-node,substrate,-p 127.0.0.1:9944:9944 -p 127.0.0.1:9933:9933,substrate-contracts-node --dev --base-path /work_path/substrate/chain_data --rpc-external --ws-external)

ui-run:
	docker run --rm -it --name polkadot-ui -e WS_URL=ws://127.0.0.1:9944 -p 80:80 --net substrate jacogr/polkadot-js-apps@sha256:43bb5b2bfab9722cdb767420c67723e8c9914c30d73fbe68a8ad31417e08876f

flipper-build:
	$(call run_ci_container,flipper-build,flipper/contract,,cargo +nightly contract build)

flipper-integ-tests:
	$(call run_ci_container,flipper-integ-tests,flipper/tests,,cargo +nightly test -- --nocapture)

subxt-cli-install:
	$(call run_ci_container,subxt-cli-install,substrate,,cargo install subxt-cli --git https://github.com/paritytech/subxt.git --rev a701d80)

gen-api:
	$(call run_ci_container,gen-api,api_metadata,,/cache/cargo/bin/subxt codegen --url http://substrate-contracts-node:9933 | rustfmt --edition=2018 --emit=stdout > api_metadata/src/lib.rs)

contracts-ci-bash:
	$(call run_ci_container,contracts-ci-bash,,,bash)
