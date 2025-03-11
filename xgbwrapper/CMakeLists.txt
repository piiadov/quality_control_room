cmake_minimum_required(VERSION 3.10)
project(xgbwrapper VERSION 0.1.0 LANGUAGES C)

set(INFERENCE_PATH ${CMAKE_BINARY_DIR}/xgb_model.json)
set(CONFIG_PATH ${CMAKE_SOURCE_DIR}/config.json)

add_compile_definitions(
    INFERENCE_PATH="${INFERENCE_PATH}"
    CONFIG_PATH="${CONFIG_PATH}"
)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Find the cJSON package
find_package(cJSON REQUIRED)

# Include the cJSON headers
include_directories(${CJSON_INCLUDE_DIRS})

# Include the XGBoost headers
include_directories(/home/vp/GitHub/xgboost/include)

# Create the shared library
add_library(xgbwrapper SHARED src/xgbwrapper.c)

# Specify the export macro
target_compile_definitions(xgbwrapper PRIVATE XGBWRAPPER_EXPORTS)

# Link the cJSON library
target_link_libraries(xgbwrapper cjson)

# Enable testing
include(CTest)
enable_testing()

# Add the test executable
add_executable(test_xgbwrapper tests/test_xgbwrapper.c)
target_link_libraries(test_xgbwrapper 
    xgbwrapper m /home/vp/GitHub/xgboost/lib/libxgboost.so
)

# Add a test
add_test(NAME TestShuffle COMMAND test_xgbwrapper test_shuffle)
add_test(NAME TestSplitData COMMAND test_xgbwrapper test_split_data)
add_test(NAME TestGenerateData COMMAND test_xgbwrapper test_generate_data)
add_test(NAME TestGenerateSimpleData COMMAND test_xgbwrapper test_generate_simple_data)
add_test(NAME TestXGBoost COMMAND test_xgbwrapper test_xgboost)

# CPack configuration
include(CPack)
