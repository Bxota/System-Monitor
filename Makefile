APP_NAME := monitor_app
BUNDLE_DIR := target/release/bundle/osx/$(APP_NAME).app

.PHONY: bundle run-bundle

bundle:
	cargo bundle --release

run-bundle: bundle
	open "$(BUNDLE_DIR)"
