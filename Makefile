# Build:      make
# Install:    sudo make install            (PREFIX=/usr/local)
# Per user:   make install PREFIX=~/.local (no root needed)
PREFIX ?= /usr/local
DESTDIR ?=
BINDIR := $(PREFIX)/bin
DATADIR := $(PREFIX)/share
APP_ID := io.github.mauneven.Beacon
DAEMON_ID := $(APP_ID).Daemon

.PHONY: all build test install uninstall clean

all: build

build:
	cargo build --locked --release

test:
	cargo test --locked

define fill_bindir
	install -d $(dir $(2))
	sed 's|@BINDIR@|$(BINDIR)|g' $(1) > $(2)
	chmod 644 $(2)
endef

install:
	install -Dm755 target/release/beacon $(DESTDIR)$(BINDIR)/beacon
	install -Dm755 target/release/beacon-daemon $(DESTDIR)$(BINDIR)/beacon-daemon
	$(call fill_bindir,data/$(APP_ID).desktop.in,$(DESTDIR)$(DATADIR)/applications/$(APP_ID).desktop)
	$(call fill_bindir,data/$(DAEMON_ID).desktop.in,$(DESTDIR)$(DATADIR)/applications/$(DAEMON_ID).desktop)
	$(call fill_bindir,data/$(DAEMON_ID).service.in,$(DESTDIR)$(DATADIR)/dbus-1/services/$(DAEMON_ID).service)
	install -Dm644 data/$(APP_ID).metainfo.xml $(DESTDIR)$(DATADIR)/metainfo/$(APP_ID).metainfo.xml
	install -Dm644 data/icons/hicolor/scalable/apps/$(APP_ID).svg $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg
	install -Dm644 data/icons/hicolor/symbolic/apps/$(APP_ID)-symbolic.svg $(DESTDIR)$(DATADIR)/icons/hicolor/symbolic/apps/$(APP_ID)-symbolic.svg
	install -Dm644 LICENSE $(DESTDIR)$(DATADIR)/licenses/beacon/LICENSE
ifeq ($(DESTDIR),)
	-gtk4-update-icon-cache -q -t -f $(DATADIR)/icons/hicolor 2>/dev/null || gtk-update-icon-cache -q -t -f $(DATADIR)/icons/hicolor 2>/dev/null
	-update-desktop-database -q $(DATADIR)/applications 2>/dev/null
endif

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/beacon $(DESTDIR)$(BINDIR)/beacon-daemon \
	      $(DESTDIR)$(DATADIR)/applications/$(APP_ID).desktop \
	      $(DESTDIR)$(DATADIR)/applications/$(DAEMON_ID).desktop \
	      $(DESTDIR)$(DATADIR)/dbus-1/services/$(DAEMON_ID).service \
	      $(DESTDIR)$(DATADIR)/metainfo/$(APP_ID).metainfo.xml \
	      $(DESTDIR)$(DATADIR)/icons/hicolor/scalable/apps/$(APP_ID).svg \
	      $(DESTDIR)$(DATADIR)/icons/hicolor/symbolic/apps/$(APP_ID)-symbolic.svg
	rm -rf $(DESTDIR)$(DATADIR)/licenses/beacon

clean:
	cargo clean
