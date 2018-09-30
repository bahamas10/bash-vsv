PREFIX ?= /usr/local

.PHONY: all
all:
	@echo 'nothing to do'

.PHONY: man
man: man/vsv.8
man/vsv.8: man/vsv.md
	md2man-roff $^ > $@

.PHONY: clean
clean:
	rm -f man/vsv.8

.PHONY: install
install:
	cp vsv $(PREFIX)/bin/vsv
	cp man/vsv.8 $(PREFIX)/share/man/man8/vsv.8

.PHONY: uninstall
uninstall:
	rm -f $(PREFIX)/bin/vsv
	rm -f $(PREFIX)/share/man/man8/vsv.8

.PHONY: check
check:
	shellcheck vsv
