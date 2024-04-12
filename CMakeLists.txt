cmake_minimum_required(VERSION 3.21)

project(ralloc)

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
