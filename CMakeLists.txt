project(ralloc)
cmake_minimum_required(VERSION 3.21)
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_BUILD_TYPE Release CACHE STRING "Build configuration (Release/Debug)")
set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release")
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

message(STATUS "Build configuration: ${CMAKE_BUILD_TYPE}")
message(STATUS "Install directory: ${CMAKE_INSTALL_PREFIX}")

set(LIBRARY_OUTPUT_PATH    ${CMAKE_BINARY_DIR}/bin)
set(EXECUTABLE_OUTPUT_PATH ${CMAKE_BINARY_DIR}/bin)

# Always include debug info
add_compile_options("$<$<COMPILE_LANGUAGE:C>:-g>")
set(CMAKE_C_EXTENSIONS ON) #...with compiler extensions like gnu++11
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

set(RALLOC_COMPILE_OPTIONS -Wall -Wno-unused-variable
    -Wno-unused-but-set-variable -Wno-unused-parameter
    -fno-omit-frame-pointer -Wno-conversion -Wno-cast-align -DRALLOC)
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(RALLOC_COMPILE_OPTIONS ${RALLOC_COMPILE_OPTIONS} -O3 -flto)
endif()

set(ralloc_src
  ${CMAKE_CURRENT_SOURCE_DIR}/src/BaseMeta.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/RegionManager.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/SizeClass.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/TCache.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/extern_val.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/ralloc.cpp
)

add_library(ralloc_cxl ${ralloc_src})
set(ralloc_inc
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/../../driver_futex/uapi/
    ${CMAKE_CURRENT_SOURCE_DIR}/../../src/driver_api/
)
target_include_directories(ralloc_cxl PUBLIC
  "$<BUILD_INTERFACE:${ralloc_inc}>"
)
target_compile_options(ralloc_cxl PUBLIC "$<$<CONFIG:Debug>:${RALLOC_COMPILE_OPTIONS};-DCXLMEM>")
target_compile_options(ralloc_cxl PUBLIC "$<$<CONFIG:Release>:${RALLOC_COMPILE_OPTIONS};-DCXLMEM>")
target_link_libraries(ralloc_cxl cxl_driver_api)