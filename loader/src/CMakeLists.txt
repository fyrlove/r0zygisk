cmake_minimum_required(VERSION 3.22.1)
project("loader")

find_package(cxx REQUIRED CONFIG)

add_definitions(-DZKSU_VERSION=\"${ZKSU_VERSION}\")
add_definitions(-DMODULE_ID=\"${MODULE_ID}\")

aux_source_directory(common COMMON_SRC_LIST)
add_library(common STATIC ${COMMON_SRC_LIST})
target_include_directories(common PRIVATE include)
target_link_libraries(common cxx::cxx log)

aux_source_directory(injector INJECTOR_SRC_LIST)
add_library(zygisk SHARED ${INJECTOR_SRC_LIST})
set_target_properties(zygisk PROPERTIES OUTPUT_NAME "r0zgk")
target_include_directories(zygisk PRIVATE include)
target_link_libraries(zygisk cxx::cxx log common lsplt_static)

add_library(zn_loader SHARED native_bridge/loader.cpp)
set_target_properties(zn_loader PROPERTIES OUTPUT_NAME "zn_loader")
target_include_directories(zn_loader PRIVATE include)
target_link_libraries(zn_loader cxx::cxx log common)

add_library(payload SHARED native_bridge/payload.cpp)
set_target_properties(payload PROPERTIES OUTPUT_NAME "payload")
target_include_directories(payload PRIVATE include)
target_link_libraries(payload cxx::cxx log)

add_subdirectory(external)
