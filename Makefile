.PHONY: all
all:
	$(MAKE) -C server
	$(MAKE) -C client
	$(MAKE) -C cli
