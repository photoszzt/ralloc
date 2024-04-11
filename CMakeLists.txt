cmake_minimum_required(VERSION 3.21)

project(ralloc)

set(ralloc_src
  ${CMAKE_CURRENT_SOURCE_DIR}/src/BaseMeta.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/RegionManager.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/SizeClass.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/TCache.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/ralloc.cpp
)

set(ralloc_inc ${CMAKE_CURRENT_SOURCE_DIR}/src)

add_library(ralloc ${ralloc_src})
target_include_directories(ralloc PRIVATE "$<BUILD_INTERFACE:${ralloc_inc}>")
target_compile_definitions(ralloc PRIVATE -DSHM_SIMULATING)
target_compile_options(ralloc PRIVATE "${PROJ_COMPILE_OPTIONS}")
target_link_options(ralloc PRIVATE "${PROJ_LINK_OPTIONS}")
