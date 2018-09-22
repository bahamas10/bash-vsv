PREFIX ?= /usr/local

.PHONY: all
all:
	@echo 'nothing to do'

.PHONY: install
install:
	cp ./vsv $(PREFIX)/bin/vsv

.PHONY: uninstall
uninstall:
	rm -f $(PREFIX)/bin/vsv

.PHONY: check
check:
	shellcheck vsv
