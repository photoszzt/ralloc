cmake_minimum_required(VERSION 3.21)

project(ralloc)

set(CMAKE_EXPORT_COMPILE_COMMANDS 1)
set(CMAKE_CXX_FLAGS_DEBUG "-O0 -g")
set(CMAKE_CXX_FLAGS_RELEASE "-O3")

add_library(ralloc
  ${CMAKE_CURRENT_SOURCE_DIR}/src/BaseMeta.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/RegionManager.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/SizeClass.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/TCache.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/ralloc.cpp
)
target_include_directories(ralloc PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)
target_compile_definitions(ralloc PRIVATE -DSHM_SIMULATING)
target_compile_options(ralloc PRIVATE "${PROJ_COMPILE_OPTIONS}")
target_link_libraries(ralloc PUBLIC numa)
target_link_options(ralloc PRIVATE "${PROJ_LINK_OPTIONS}")

add_library(ralloc_dynamic SHARED
    ${CMAKE_CURRENT_SOURCE_DIR}/src/ralloc_dynamic.cpp
)
target_compile_definitions(ralloc_dynamic PRIVATE -DSHM_SIMULATING)
target_compile_options(ralloc_dynamic PRIVATE -ftls-model=initial-exec)
target_link_libraries(ralloc_dynamic PRIVATE ralloc numa)

add_library(ralloc_static STATIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src/ralloc_static.cpp
)
target_compile_definitions(ralloc_static PRIVATE -DSHM_SIMULATING)
target_compile_options(ralloc_static PRIVATE)
target_link_libraries(ralloc_static PRIVATE ralloc numa)
