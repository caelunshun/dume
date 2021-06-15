cmake_minimum_required(VERSION 3.14)
project(dume)

set(SOURCES src/dume.cpp)

set(CMAKE_CXX_STANDARD 20)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

include(FetchContent)

FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/AndrewGaspar/corrosion.git
        GIT_TAG origin/master
)

FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH renderer/Cargo.toml)

find_package(glfw3 REQUIRED)

add_library(dume ${SOURCES})
target_include_directories(dume PUBLIC include/ renderer/include/)
target_link_libraries(dume PUBLIC glfw dume-renderer ${CONAN_LIBS})

add_executable(dume_example example/sprite.cpp)
target_link_libraries(dume_example dume)
