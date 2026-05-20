cmake_minimum_required(VERSION 3.16)
project(rat
    VERSION 0.2.0
    LANGUAGES C
)

option(BUILD_RAT_CLI "Build the rat_cli executable" ON)
option(BUILD_TESTS   "Build C tests"                 ON)
option(DEBUGC        "Define DEBUG when compiling C" ON)

if(WIN32)
    message(FATAL_ERROR "Unsupported platform: Windows")
endif()

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_VISIBILITY_PRESET hidden)

# ---  rat static library  ---

set(RAT_LIB_SRC
    src/rat_core.c
    src/rat_packets.c
    src/rat_utils.c
)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    list(APPEND RAT_LIB_SRC src/platforms/rat_linux.c)
elseif(CMAKE_SYSTEM_NAME MATCHES "Darwin|FreeBSD|OpenBSD|NetBSD|DragonFly")
    list(APPEND RAT_LIB_SRC src/platforms/rat_bsd.c)
else()
    message(FATAL_ERROR "Unsupported platform: ${CMAKE_SYSTEM_NAME}")
endif()

add_library(rat STATIC ${RAT_LIB_SRC})
target_include_directories(rat PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)

if(DEBUGC)
    target_compile_definitions(rat PRIVATE DEBUG)
endif()

# ---  rat_cli executable  ---

if(BUILD_RAT_CLI)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(READLINE REQUIRED IMPORTED_TARGET readline)
    pkg_check_modules(LIBUV    REQUIRED IMPORTED_TARGET libuv)

    add_executable(rat_cli cli/rat_cli.c)
    target_compile_definitions(rat_cli PRIVATE
        RAT_VERSION="v${PROJECT_VERSION}"
    )
    target_link_libraries(rat_cli PRIVATE
        rat
        PkgConfig::READLINE
        PkgConfig::LIBUV
    )
    install(TARGETS rat_cli RUNTIME DESTINATION bin)
endif()

# ---  C tests  ---

if(BUILD_TESTS)
    enable_testing()
    add_executable(test_rat_cap tests/test_rat_cap.c)
    target_link_libraries(test_rat_cap PRIVATE rat)
    add_test(NAME test_rat_cap COMMAND test_rat_cap)
endif()
