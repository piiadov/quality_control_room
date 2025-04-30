cmake_minimum_required(VERSION 3.10)
project(xgbwrapper VERSION 0.1.0 LANGUAGES C)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Include the XGBoost headers
include_directories(/home/vp/xgboost/include)

# Create the shared library
add_library(xgbwrapper SHARED src/xgbwrapper.c)

# Specify the export macro
target_compile_definitions(xgbwrapper PRIVATE XGBWRAPPER_EXPORTS)

# Enable testing
include(CTest)
enable_testing()

# Add the test executable
add_executable(test_xgbwrapper tests/test_xgbwrapper.c)
target_link_libraries(test_xgbwrapper 
    xgbwrapper m /home/vp/xgboost/lib/libxgboost.so
)

# Add a test
add_test(NAME TestShuffle COMMAND test_xgbwrapper test_shuffle)
add_test(NAME TestSplitData COMMAND test_xgbwrapper test_split_data)
add_test(NAME TestGenerateData COMMAND test_xgbwrapper test_generate_data)
add_test(NAME TestGenerateSimpleData COMMAND test_xgbwrapper test_generate_simple_data)
add_test(NAME TestXGBoost COMMAND test_xgbwrapper test_xgboost)

set(COPY_DESTINATION /home/vp/quality_control_room/lib)
add_custom_command(TARGET xgbwrapper POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:xgbwrapper> ${COPY_DESTINATION}
)
