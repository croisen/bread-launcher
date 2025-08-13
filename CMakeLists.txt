cmake_minimum_required(VERSION 3.30)
file(STRINGS src/config.h __raw_version_string REGEX "BREAD_LAUNCHER_VERSION \"([0-9]+\\.[0-9]+\\.[0-9]+)\"")
string(REGEX MATCH "([0-9]+\\.?)+" __bread_launcher_version "${__raw_version_string}")

project(bread-launcher
    VERSION ${__bread_launcher_version}
    LANGUAGES CXX
    DESCRIPTION "A shady minecraft launcher"
    HOMEPAGE_URL "https://github.com/croisen/bread-launcher"
)

find_package(nghttp3 REQUIRED)
find_package(ngtcp2 REQUIRED)
find_package(CURL REQUIRED)
find_package(nlohmann_json REQUIRED)
find_package(SDL3 REQUIRED)

add_executable(${PROJECT_NAME}
    src/main.cpp

    3rd_party/imgui/imgui.cpp
    3rd_party/imgui/imgui_demo.cpp
    3rd_party/imgui/imgui_draw.cpp
    3rd_party/imgui/imgui_tables.cpp
    3rd_party/imgui/imgui_widgets.cpp
    3rd_party/imgui/backends/imgui_impl_sdl3.cpp
    3rd_party/imgui/backends/imgui_impl_sdlrenderer3.cpp
)

target_include_directories(${PROJECT_NAME} PRIVATE
    3rd_party/imgui
    3rd_party/imgui/backends
)

target_link_directories(${PROJECT_NAME} PRIVATE
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
)

target_link_libraries(${PROJECT_NAME} PRIVATE
    ngtcp2::ngtcp2_static
    nghttp3::nghttp3_static
    OpenSSL::SSL
    OpenSSL::Crypto
    nlohmann_json::nlohmann_json
    SDL3::SDL3
)

install(TARGETS ${PROJECT_NAME})
