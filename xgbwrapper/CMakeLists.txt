cmake_minimum_required(VERSION 3.14)
project(xgbwrapper 
    VERSION 0.4.0 
    DESCRIPTION "XGBoost C wrapper library for quality control applications"
    LANGUAGES C
)

# ==============================================================================
# Build Options
# ==============================================================================
option(XGBWRAPPER_BUILD_TESTS "Build test executables" ON)
option(XGBWRAPPER_INSTALL "Generate install target" ON)

# ==============================================================================
# Paths Configuration
# ==============================================================================
# Get the project root directory (parent of xgbwrapper)
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/.." ABSOLUTE)

set(XGBOOST_ROOT "${PROJECT_ROOT}/xgboost" CACHE PATH "Path to XGBoost source")
set(LIB_OUTPUT_DIR "${PROJECT_ROOT}/lib" CACHE PATH "Output directory for libraries")

# ==============================================================================
# Find Dependencies
# ==============================================================================
# XGBoost headers
if(EXISTS "${XGBOOST_ROOT}/include")
    set(XGBOOST_INCLUDE_DIR "${XGBOOST_ROOT}/include")
else()
    message(FATAL_ERROR "XGBoost headers not found at ${XGBOOST_ROOT}/include. "
                        "Please clone XGBoost: git clone --recurse-submodules https://github.com/dmlc/xgboost.git")
endif()

# XGBoost library
find_library(XGBOOST_LIBRARY 
    NAMES xgboost
    PATHS "${LIB_OUTPUT_DIR}" "${XGBOOST_ROOT}/lib"
    NO_DEFAULT_PATH
)
if(NOT XGBOOST_LIBRARY)
    message(FATAL_ERROR "libxgboost.so not found. Please build XGBoost first:\n"
                        "  cd ${XGBOOST_ROOT} && mkdir build && cd build\n"
                        "  cmake .. -DBUILD_SHARED_LIBS=ON && make -j\$(nproc)\n"
                        "  cp lib/libxgboost.so ${LIB_OUTPUT_DIR}/")
endif()

message(STATUS "XGBoost include: ${XGBOOST_INCLUDE_DIR}")
message(STATUS "XGBoost library: ${XGBOOST_LIBRARY}")

# ==============================================================================
# Compiler Settings
# ==============================================================================
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Warning flags
if(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# ==============================================================================
# Main Library Target
# ==============================================================================
add_library(xgbwrapper SHARED
    src/xgbwrapper.c
)

target_include_directories(xgbwrapper
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/src>
        $<INSTALL_INTERFACE:include>
    PRIVATE
        ${XGBOOST_INCLUDE_DIR}
)

target_link_libraries(xgbwrapper
    PRIVATE
        ${XGBOOST_LIBRARY}
        m  # math library
)

# Export symbols
target_compile_definitions(xgbwrapper
    PRIVATE
        XGBWRAPPER_EXPORTS
)

# Set library output directory
set_target_properties(xgbwrapper PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
    VERSION ${PROJECT_VERSION}
    SOVERSION ${PROJECT_VERSION_MAJOR}
)

# ==============================================================================
# Post-build: Copy to project lib directory
# ==============================================================================
add_custom_command(TARGET xgbwrapper POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory "${LIB_OUTPUT_DIR}"
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:xgbwrapper> "${LIB_OUTPUT_DIR}/"
    COMMENT "Copying libxgbwrapper to ${LIB_OUTPUT_DIR}"
)

# ==============================================================================
# Tests
# ==============================================================================
if(XGBWRAPPER_BUILD_TESTS)
    include(CTest)
    enable_testing()
    
    add_executable(test_xgbwrapper
        tests/test_xgbwrapper.c
    )
    
    target_include_directories(test_xgbwrapper
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${XGBOOST_INCLUDE_DIR}
    )
    
    target_link_libraries(test_xgbwrapper
        PRIVATE
            xgbwrapper
            ${XGBOOST_LIBRARY}
            m
    )
    
    # Set RPATH for test executable to find libraries
    set_target_properties(test_xgbwrapper PROPERTIES
        BUILD_RPATH "${LIB_OUTPUT_DIR};${CMAKE_BINARY_DIR}/lib"
        INSTALL_RPATH "${LIB_OUTPUT_DIR}"
    )
    
    # Register tests
    add_test(NAME test_shuffle COMMAND test_xgbwrapper test_shuffle)
    add_test(NAME test_split_data COMMAND test_xgbwrapper test_split_data)
    add_test(NAME test_generate_data COMMAND test_xgbwrapper test_generate_data)
    add_test(NAME test_generate_simple_data COMMAND test_xgbwrapper test_generate_simple_data)
    add_test(NAME test_xgboost COMMAND test_xgbwrapper test_xgboost)
    
    # Set test environment
    set_tests_properties(test_shuffle test_split_data test_generate_data 
                         test_generate_simple_data test_xgboost
        PROPERTIES
            ENVIRONMENT "LD_LIBRARY_PATH=${LIB_OUTPUT_DIR}:$ENV{LD_LIBRARY_PATH}"
    )
endif()

# ==============================================================================
# Installation
# ==============================================================================
if(XGBWRAPPER_INSTALL)
    include(GNUInstallDirs)
    
    install(TARGETS xgbwrapper
        EXPORT xgbwrapperTargets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
    
    install(FILES src/xgbwrapper.h
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )
    
    # Export targets
    install(EXPORT xgbwrapperTargets
        FILE xgbwrapperTargets.cmake
        NAMESPACE xgbwrapper::
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/xgbwrapper
    )
endif()

# ==============================================================================
# Summary
# ==============================================================================
message(STATUS "")
message(STATUS "xgbwrapper ${PROJECT_VERSION} configuration:")
message(STATUS "  Build tests:     ${XGBWRAPPER_BUILD_TESTS}")
message(STATUS "  Install target:  ${XGBWRAPPER_INSTALL}")
message(STATUS "  Library output:  ${LIB_OUTPUT_DIR}")
message(STATUS "")
