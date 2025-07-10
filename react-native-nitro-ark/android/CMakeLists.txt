project(NitroArk)
cmake_minimum_required(VERSION 3.9.0)

set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_CXX_STANDARD 20)

# Define paths
set(CPP_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../cpp")
set(ANDROID_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
set(LIBS_DIR "${ANDROID_DIR}/src/main/jniLibs/${ANDROID_ABI}")

# Check for required files
set(BARK_LIB "${LIBS_DIR}/libbark_cpp.a")
set(BARK_CXX_LIB "${LIBS_DIR}/libcxxbridge1.a")

if(NOT EXISTS "${BARK_LIB}")
  message(FATAL_ERROR "libbark_cpp.a not found at: ${BARK_LIB}")
endif()

if(NOT EXISTS "${BARK_CXX_LIB}")
    message(FATAL_ERROR "libcxxbridge1.a not found at: ${BARK_CXX_LIB}")
endif()

message(STATUS "Using libraries from: ${LIBS_DIR}")

# Define C++ library and add all sources
add_library(
  ${PROJECT_NAME}
  SHARED
  src/main/cpp/cpp-adapter.cpp
  ../cpp/HybridArk.cpp
)

# Include directories
target_include_directories(${PROJECT_NAME}
    PRIVATE
    ${CPP_DIR}
    ${CPP_DIR}/generated
    ${ANDROID_DIR}/src/main/cpp
    ${LIBS_DIR}
)

# Find Android log library
find_library(LOG_LIB log)

# Create imported targets for external libraries
add_library(bark_lib STATIC IMPORTED)
set_target_properties(bark_lib PROPERTIES
    IMPORTED_LOCATION "${BARK_LIB}"
)

add_library(bark_cxx_lib STATIC IMPORTED)
set_target_properties(bark_cxx_lib PROPERTIES
    IMPORTED_LOCATION "${BARK_CXX_LIB}"
)

# Additional compiler flags
target_compile_options(${PROJECT_NAME}
    PRIVATE
    -fexceptions
    -frtti
)

# Include autolinking
include(${CMAKE_SOURCE_DIR}/../nitrogen/generated/android/NitroArk+autolinking.cmake)

# Add our additional libraries after autolinking
target_link_libraries(
    ${PROJECT_NAME}
    bark_lib
    bark_cxx_lib
    ${LOG_LIB}
    dl
    m
    android
)
