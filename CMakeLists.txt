cmake_minimum_required(VERSION 3.14)
project(riposte)

set(SOURCES src/dume.cpp)

add_library(riposte ${SOURCES})
target_include_directories(riposte PUBLIC include/)
