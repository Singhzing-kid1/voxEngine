cmake_minimum_required(VERSION 3.10)

project(voxEngine)

set(OpenGL_GL_PREFERENCE GLVND)
cmake_policy(SET CMP0167 NEW)

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-Wa,-mbig-obj)
endif()

include_directories(include)

file(GLOB HEADER_FILES "./include/*.hpp")
file(GLOB SOURCE_FILES "./src/*.cpp")

file(GLOB SHADER_SOURCE_FILES "./shaders/*.glsl")
file(COPY ${SHADER_SOURCE_FILES} DESTINATION "shaders")

add_executable(main ${HEADER_FILES} ${SOURCE_FILES})

find_package(glm REQUIRED)
target_link_libraries(main PRIVATE glm::glm)

find_package(OpenGL REQUIRED)
target_link_libraries(main PRIVATE OpenGL::GL)

find_package(GLEW REQUIRED)
target_link_libraries(main PRIVATE GLEW::GLEW)

find_package(OpenVDB REQUIRED)
target_link_libraries(main PRIVATE OpenVDB::openvdb)

find_package(SDL2 CONFIG REQUIRED)
target_link_libraries(main
    PRIVATE
    $<TARGET_NAME_IF_EXISTS:SDL2::SDL2main>
    $<IF:$<TARGET_EXISTS:SDL2::SDL2>,SDL2::SDL2,SDL2::SDL2-static>
)

find_package(Bullet REQUIRED)
target_link_libraries(main PRIVATE ${BULLET_LIBRARIES})
target_include_directories(main PRIVATE ${BULLET_INCLUDE_DIRS})

find_package(OpenMP REQUIRED)
if(OpenMP_CXX_FOUND)
    target_link_libraries(main PUBLIC OpenMP::OpenMP_CXX)
endif()