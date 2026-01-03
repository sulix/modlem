# If rustc is not overridden, set it
RUSTC ?= rustc

BIN = modlem

SOURCES = src/modlem.rs \
	  src/main_dat.rs \
	  src/case_sensitivity.rs \
	  src/binary_io.rs \
	  src/parser.rs \
	  src/planar_bmp.rs

$(BIN): $(SOURCES)
	$(RUSTC) -o $@ $<

.pseudo: clean

clean:
	rm $(BIN)
