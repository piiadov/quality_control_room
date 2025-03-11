set(CPACK_PACKAGE_NAME ${PROJECT_NAME})
set(CPACK_PACKAGE_VERSION ${PROJECT_VERSION})
set(CPACK_PACKAGE_DESCRIPTION "Wrapper for reference XGBoost convenient to use in Rust")
set(CPACK_GENERATOR "ZIP;TGZ")
set(CPACK_PACKAGE_VENDOR "Vasilii Piiadov")
set(CPACK_PACKAGE_CONTACT "piyadov@alumni.usp.br")
set(CPACK_PACKAGE_DIRECTORY "${CMAKE_BINARY_DIR}/package")

include(CPack)
