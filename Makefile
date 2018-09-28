PREFIX ?= /usr/local

.PHONY: all
all:
	@echo 'nothing to do'

.PHONY: man
man: man/vsv.8
man/vsv.8: man/vsv.md
	md2man-roff $^ > $@


.PHONY: install
install:
	cp vsv $(PREFIX)/bin/vsv

.PHONY: uninstall
uninstall:
	rm -f $(PREFIX)/bin/vsv

.PHONY: check
check:
	shellcheck vsv
