.PHONY: all build live help

BOOK=kubook

all: build

build:
	@echo '=== Build ==='
	mdbook build $(BOOK)

live:
	@echo '=== Serve with live reload ==='
	mdbook serve --watcher native $(BOOK)

help:
	@echo "Usage: make [target]"
	@echo "Targets:"
	@echo "  all        - Build"
	@echo "  build      - Build the book"
	@echo "  live       - Serve the book with live reload"
	@echo "  help       - Show this help message"
