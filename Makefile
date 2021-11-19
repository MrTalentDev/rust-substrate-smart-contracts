SHELL=/bin/sh

mkfile_path = $(abspath $(lastword $(MAKEFILE_LIST)))
current_dir = $(patsubst %/,%,$(dir $(mkfile_path)))
contracts_ci_linux_tag = 3582091f-20211118
contracts_ci_linux_image = paritytech/contracts-ci-linux
id_u = $(shell id -u)
id_g = $(shell id -g)

define run_ci_container =
	docker run --rm -it --user $(id_u):$(id_g) -w /work_path -v $(current_dir)/$(1):/work_path -v $(current_dir)/cache/:/cache/ -e CARGO_HOME=/cache/cargo/ -e SCCACHE_DIR=/cache/sccache/ $(contracts_ci_linux_image):$(contracts_ci_linux_tag) $(2)
endef

new-contract:
ifdef contract_name
	$(call run_ci_container,,cargo contract new $(contract_name))
else
	$(error No contract_name argument provided)
endif

node-install-force:
	$(call run_ci_container,substrate,cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --force --locked)

node-run:
	$(call run_ci_container,substrate,substrate-contracts-node --dev --base-path /work_path/chain_data)

ui-run:
	docker run --rm -it --name polkadot-ui -e WS_URL=ws://127.0.0.1:9944 -p 80:80 jacogr/polkadot-js-apps@sha256:43bb5b2bfab9722cdb767420c67723e8c9914c30d73fbe68a8ad31417e08876f

flipper-build:
	$(call run_ci_container,flipper,cargo contract build)
