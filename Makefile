PROJECT           = bread-launcher
PROJECT_ROOT      = $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
INSTALL_PREFIX    ?= /usr/local
ARTIFACTS         ?= /tmp/$(PROJECT)
COMPILER_PREFIX   ?=
CFLAGS            = -Wall -O3 --std=c++23 \
					-I$(INSTALL_PREFIX)/include \
					-I$(PROJECT_ROOT)/3rd_party/imgui \
					-I$(PROJECT_ROOT)/3rd_party/imgui/backends


rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
SOURCES           = $(call rwildcard,$(PROJECT_ROOT)/src,*.cpp) \
					$(wildcard $(PROJECT_ROOT)/3rd_party/imgui/*.cpp) \
					$(PROJECT_ROOT)/3rd_party/imgui/backends/imgui_impl_sdl3.cpp \
					$(PROJECT_ROOT)/3rd_party/imgui/backends/imgui_impl_sdlrenderer3.cpp
OBJECTS0          = $(patsubst $(PROJECT_ROOT)/src/%.cpp,$(ARTIFACTS)/$(PROJECT)/%.o,$(SOURCES))
OBJECTS           = $(patsubst $(PROJECT_ROOT)/3rd_party/%.cpp,$(ARTIFACTS)/$(PROJECT)/%.o,$(OBJECTS0))

install: $(ARTIFACTS)/$(PROJECT)/$(PROJECT)
	install $< $(INSTALL_PREFIX)/bin

$(ARTIFACTS)/$(PROJECT)/$(PROJECT): $(OBJECTS) | $(INSTALL_PREFIX)
	$(COMPILER_PREFIX)c++ \
		$(CFLAGS) \
		-o $@ \
		$^ \
		-Wl,-Bstatic \
		-L$(INSTALL_PREFIX)/lib \
		-lSDL3 \
		-lz \
		-lbrotlicommon \
		-lbrotlienc \
		-lbrotlidec \
		-lzstd \
		-lssl \
		-lcrypto \
		-Wl,-Bdynamic

$(ARTIFACTS)/$(PROJECT)/%.o: $(PROJECT_ROOT)/src/%.cpp
	if ! [ -d "$(dir $@)" ]; then mkdir --parents "$(dir $@)"; fi
	$(COMPILER_PREFIX)c++ \
		$(CFLAGS) \
		-c $< \
		-o $@

$(ARTIFACTS)/$(PROJECT)/%.o: $(PROJECT_ROOT)/3rd_party/%.cpp
	if ! [ -d "$(dir $@)" ]; then mkdir --parents "$(dir $@)"; fi
	$(COMPILER_PREFIX)c++ \
		$(CFLAGS) \
		-c $< \
		-o $@

$(INSTALL_PREFIX):
	mkdir --parents "$@"
