# modlem: A graphics importer/exporter for Lemmings
# Copyright (C) 2022–2026 David Gow <david@davidgow.net>
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.

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
