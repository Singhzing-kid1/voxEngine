cmake_minimum_required(VERSION 3.10)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address -fno-omit-frame-pointer")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fsanitize=address")


project(voxEngine)

set(OpenGL_GL_PREFERENCE GLVND)
cmake_policy(SET CMP0167 NEW)

include_directories(include)

file(GLOB HEADER_FILES "./include/*.hpp")
file(GLOB SOURCE_FILES "./src/*.cpp")

file(GLOB SHADER_VERTEX_SOURCE_FILES "./shaders/*.vert")
file(GLOB SHADER_FRAGMENT_SOURCE_FILES "./shaders/*.frag")
file(GLOB SHADER_COMPUTE_SOURCE_FILES "./shaders/*.comp")
file(COPY ${SHADER_VERTEX_SOURCE_FILES} DESTINATION "shaders")
file(COPY ${SHADER_FRAGMENT_SOURCE_FILES} DESTINATION "shaders")
file(COPY ${SHADER_COMPUTE_SOURCE_FILES} DESTINATION "shaders")

file(GLOB FONT_SOURCE_FILES "./fonts/*.ttf")
file(COPY ${FONT_SOURCE_FILES} DESTINATION "fonts")

add_executable(main ${HEADER_FILES} ${SOURCE_FILES})

find_package(glm REQUIRED)
target_link_libraries(main PRIVATE glm::glm)

find_package(OpenGL REQUIRED)
target_link_libraries(main PRIVATE OpenGL::GL)

find_package(GLEW REQUIRED)
target_link_libraries(main PRIVATE GLEW::GLEW)

find_package(SDL2 CONFIG REQUIRED)
target_link_libraries(main
    PRIVATE
    $<TARGET_NAME_IF_EXISTS:SDL2::SDL2main>
    $<IF:$<TARGET_EXISTS:SDL2::SDL2>,SDL2::SDL2,SDL2::SDL2-static>
)

find_package(SDL2_ttf REQUIRED)
target_link_libraries(main PRIVATE SDL2_ttf::SDL2_ttf)

find_package(Bullet REQUIRED)
target_link_libraries(main PRIVATE ${BULLET_LIBRARIES})
target_include_directories(main PRIVATE ${BULLET_INCLUDE_DIRS})

find_package(OpenMP REQUIRED)
if(OpenMP_CXX_FOUND)
    target_link_libraries(main PUBLIC OpenMP::OpenMP_CXX)
endif()